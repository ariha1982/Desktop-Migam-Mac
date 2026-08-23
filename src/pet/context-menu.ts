import { invoke } from "@tauri-apps/api/core";
import { cursorPosition } from "@tauri-apps/api/window";

export async function attachPetContextMenu(target: HTMLElement): Promise<() => void> {
  const openMenu = (event: MouseEvent): void => {
    event.preventDefault();
    event.stopPropagation();
    void cursorPosition().then((position) =>
      invoke("show_pet_context_menu", {
        x: Math.round(position.x),
        y: Math.round(position.y),
      }),
    );
  };
  target.addEventListener("contextmenu", openMenu);

  return () => {
    target.removeEventListener("contextmenu", openMenu);
  };
}
