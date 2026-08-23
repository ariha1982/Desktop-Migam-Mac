import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import pullStripUrl from "../../images/characters/gamjabot/extra/photo-delivery/gamjabot-pull-strip.png";
import hyperVPhotoUrl from "../../images/characters/gamjabot/extra/photo-delivery/photos/hyper-v.png";
import notepadPhotoUrl from "../../images/characters/gamjabot/extra/photo-delivery/photos/notepad.png";
import visualStudioPhotoUrl from "../../images/characters/gamjabot/extra/photo-delivery/photos/visual-studio.png";
import windbgPhotoUrl from "../../images/characters/gamjabot/extra/photo-delivery/photos/windbg.png";

const PULL_DURATION_MILLISECONDS = 18_000;
const PET_LEAVE_DURATION_MILLISECONDS = 3_500;
const FIRST_DELIVERY_MINIMUM_MILLISECONDS = 120_000;
const DELIVERY_DELAY_RANGE_MILLISECONDS = 120_000;

const photoUrls = [
  visualStudioPhotoUrl,
  windbgPhotoUrl,
  hyperVPhotoUrl,
  notepadPhotoUrl,
];

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.addEventListener("load", () => resolve(image), { once: true });
    image.addEventListener("error", () => reject(new Error("pull sprite could not be loaded")), {
      once: true,
    });
    image.src = url;
  });
}

function isLightBackground(data: Uint8ClampedArray, pixel: number): boolean {
  const offset = pixel * 4;
  const red = data[offset];
  const green = data[offset + 1];
  const blue = data[offset + 2];
  const maximum = Math.max(red, green, blue);
  const minimum = Math.min(red, green, blue);
  return minimum >= 218 && maximum - minimum <= 34;
}

async function transparentPullStrip(): Promise<string> {
  const source = await loadImage(pullStripUrl);
  const canvas = document.createElement("canvas");
  canvas.width = source.naturalWidth;
  canvas.height = source.naturalHeight;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) throw new Error("canvas is unavailable");
  context.drawImage(source, 0, 0);

  const image = context.getImageData(0, 0, canvas.width, canvas.height);
  const pixelCount = canvas.width * canvas.height;
  const visited = new Uint8Array(pixelCount);
  const queue = new Int32Array(pixelCount);
  let head = 0;
  let tail = 0;
  const add = (pixel: number): void => {
    if (visited[pixel] !== 0 || !isLightBackground(image.data, pixel)) return;
    visited[pixel] = 1;
    queue[tail++] = pixel;
  };

  for (let x = 0; x < canvas.width; x += 1) {
    add(x);
    add((canvas.height - 1) * canvas.width + x);
  }
  for (let y = 0; y < canvas.height; y += 1) {
    add(y * canvas.width);
    add(y * canvas.width + canvas.width - 1);
  }
  while (head < tail) {
    const pixel = queue[head++];
    const x = pixel % canvas.width;
    const y = Math.floor(pixel / canvas.width);
    image.data[pixel * 4 + 3] = 0;
    if (x > 0) add(pixel - 1);
    if (x + 1 < canvas.width) add(pixel + 1);
    if (y > 0) add(pixel - canvas.width);
    if (y + 1 < canvas.height) add(pixel + canvas.width);
  }
  context.putImageData(image, 0, 0);

  const blob = await new Promise<Blob>((resolve, reject) =>
    canvas.toBlob((result) => result ? resolve(result) : reject(new Error("sprite conversion failed")), "image/png"),
  );
  return URL.createObjectURL(blob);
}

export function startPhotoDeliveryScheduler(): () => void {
  let active = true;
  let timeout = 0;
  const schedule = (): void => {
    const delay =
      FIRST_DELIVERY_MINIMUM_MILLISECONDS + Math.random() * DELIVERY_DELAY_RANGE_MILLISECONDS;
    timeout = window.setTimeout(() => {
      if (!active) return;
      void invoke<boolean>("start_photo_delivery").finally(schedule);
    }, delay);
  };
  schedule();
  return () => {
    active = false;
    window.clearTimeout(timeout);
  };
}

export async function mountPhotoDelivery(container: HTMLElement): Promise<() => void> {
  container.innerHTML = `
    <main class="photo-delivery-stage" aria-live="polite">
      <section class="photo-delivery-rig" aria-label="감자펫의 사진 배달">
        <div class="photo-delivery-pet" aria-hidden="true"></div>
        <figure class="photo-delivery-card">
          <img alt="감자펫이 가져온 사진" />
          <button type="button" aria-label="사진 닫기">×</button>
        </figure>
      </section>
    </main>`;
  const rig = container.querySelector<HTMLElement>(".photo-delivery-rig")!;
  const pet = container.querySelector<HTMLElement>(".photo-delivery-pet")!;
  const photo = container.querySelector<HTMLImageElement>(".photo-delivery-card img")!;
  const close = container.querySelector<HTMLButtonElement>(".photo-delivery-card button")!;
  const spriteUrlPromise = transparentPullStrip();
  let phase: "idle" | "delivering" | "leaving" | "settled" = "idle";
  let leaveTimer = 0;
  let route: Animation | null = null;

  const reset = (): void => {
    window.clearTimeout(leaveTimer);
    route?.cancel();
    route = null;
    phase = "idle";
    rig.classList.remove("delivering", "delivered", "settled", "from-left", "from-right");
    rig.style.removeProperty("transform");
  };

  const finish = (): void => {
    reset();
    void invoke("finish_photo_delivery");
  };

  const settlePhoto = (): void => {
    const bounds = container.querySelector<HTMLElement>(".photo-delivery-card")!.getBoundingClientRect();
    void invoke("settle_photo_delivery", {
      left: bounds.left,
      top: bounds.top,
      width: bounds.width,
      height: bounds.height,
    }).catch(finish);
  };

  const leavePet = (): void => {
    if (phase !== "delivering") return;
    phase = "leaving";
    rig.style.transform = `translate(${rig.dataset.targetX}px, ${rig.dataset.targetY}px)`;
    route?.cancel();
    route = null;
    rig.classList.remove("delivering");
    rig.classList.add("delivered");
    leaveTimer = window.setTimeout(settlePhoto, PET_LEAVE_DURATION_MILLISECONDS);
  };

  const pullKeyframes = (startX: number, targetX: number, y: number): Keyframe[] => {
    const stops = [0, 0.09, 0.075, 0.2, 0.185, 0.34, 0.325, 0.5, 0.485, 0.66, 0.645, 0.82, 0.805, 1];
    return stops.map((progress, index) => ({
      offset: index / (stops.length - 1),
      transform: `translate(${startX + (targetX - startX) * progress}px, ${y}px)`,
      easing: index % 2 === 0 ? "cubic-bezier(.18,.72,.24,1)" : "ease-out",
    }));
  };

  const start = (): void => {
    if (phase !== "idle") return;
    phase = "delivering";
    const selectedPhoto = photoUrls[Math.floor(Math.random() * photoUrls.length)];
    photo.src = selectedPhoto;
    void Promise.all([spriteUrlPromise, photo.decode()]).then(([spriteUrl]) => {
      if (phase !== "delivering") return;
      pet.style.backgroundImage = `url("${spriteUrl}")`;
      const maximumWidth = Math.min(520, window.innerWidth * 0.52);
      const maximumHeight = Math.min(420, window.innerHeight * 0.58);
      const scale = Math.min(maximumWidth / photo.naturalWidth, maximumHeight / photo.naturalHeight);
      const photoWidth = Math.max(300, Math.round(photo.naturalWidth * scale));
      const photoHeight = Math.max(240, Math.round(photo.naturalHeight * scale));
      const margin = 32;
      const sideInset = Math.min(90, Math.max(0, (window.innerWidth - photoWidth - margin * 2) * 0.12));
      const comesFromLeft = Math.random() < 0.5;
      const targetPhotoX = comesFromLeft
        ? margin + Math.random() * sideInset
        : window.innerWidth - photoWidth - margin - Math.random() * sideInset;
      const targetPhotoY = margin + Math.random() * Math.max(0, window.innerHeight - photoHeight - margin * 2);
      const targetX = comesFromLeft ? targetPhotoX : targetPhotoX - 123;
      const targetY = targetPhotoY + photoHeight - 200;
      const startX = comesFromLeft ? -(photoWidth + 183) : window.innerWidth + 60;

      rig.style.setProperty("--photo-width", `${photoWidth}px`);
      rig.style.setProperty("--photo-height", `${photoHeight}px`);
      rig.dataset.targetX = String(targetX);
      rig.dataset.targetY = String(targetY);
      rig.classList.remove("delivering", "delivered", "settled", "from-left", "from-right");
      rig.classList.add("delivering", comesFromLeft ? "from-left" : "from-right");
      route = rig.animate(pullKeyframes(startX, targetX, targetY), {
        duration: PULL_DURATION_MILLISECONDS,
        fill: "forwards",
      });
      void route.finished.then(leavePet).catch(() => undefined);
    }).catch(() => {
      finish();
    });
  };

  const settle = (): void => {
    phase = "settled";
    rig.style.removeProperty("transform");
    rig.classList.remove("delivering", "delivered");
    rig.classList.add("settled");
  };

  const unlisten = await listen("photo://deliver", start);
  const unlistenSettled = await listen("photo://settled", settle);
  const unlistenReset = await listen("photo://reset", reset);
  const unlistenEmergency = await listen("app://emergency-stopped", reset);
  close.addEventListener("click", finish);
  const spriteUrl = await spriteUrlPromise.catch(() => "");
  return () => {
    window.clearTimeout(leaveTimer);
    close.removeEventListener("click", finish);
    unlisten();
    unlistenSettled();
    unlistenReset();
    unlistenEmergency();
    if (spriteUrl) URL.revokeObjectURL(spriteUrl);
  };
}
