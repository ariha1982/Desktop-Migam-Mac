import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { invokeWhenReady } from "../tauri/invoke-when-ready";

interface TodoItem { id: string; text: string; createdAt: string; completedAt: string | null; }
interface FocusTodoLink { todoId: string; titleAtStart: string; }
interface TodoSnapshot {
  schemaVersion: number;
  items: TodoItem[];
  selectedTodoId: string | null;
  allCompletedCelebrated: boolean;
  activeFocusTodo: FocusTodoLink | null;
  pendingFocusTodo: FocusTodoLink | null;
}

export async function mountTodo(container: HTMLElement): Promise<() => void> {
  container.innerHTML = `
    <main class="todo-app">
      <section class="todo-debug-document">
        <div class="todo-pane-title"><span>DesktopPet.Todo</span></div>
        <div class="todo-command-line"><span>0:000&gt;</span><span>.todo /today /local</span><span class="todo-caret">_</span></div>
        <div class="todo-app-header"><div><span class="todo-kicker">GAMJABOT TASK MANAGER</span><h1>오늘 할 일</h1></div><strong id="todo-summary">0 / 0</strong></div>
        <fieldset class="todo-focus-card" aria-label="현재 집중 항목"><legend>Focus Target</legend><span>지금 집중할 일</span><strong id="todo-current">선택된 할 일 없음</strong><button id="todo-start-focus" type="button">집중 시작</button></fieldset>
        <form class="todo-quick-add"><label for="todo-input">New Task</label><div><input id="todo-input" type="text" maxlength="200" autocomplete="off" placeholder="할 일을 입력하세요" aria-label="새 할 일" /><button type="submit">추가</button></div></form>
        <div class="todo-section-heading"><strong>Today</strong><button id="todo-toggle-completed" type="button">완료 숨기기</button></div>
        <section id="todo-list" class="todo-app-list" aria-live="polite"></section>
        <section id="todo-focus-result" class="todo-result-card" hidden><strong>집중을 마쳤어요</strong><p id="todo-focus-result-title"></p><div><button data-focus-action="complete" type="button">완료하기</button><button data-focus-action="continue" type="button">계속하기</button><button data-focus-action="next" type="button">다음 집중</button></div></section>
        <div id="todo-celebration" class="todo-app-celebration" hidden>다 끝냈다! 오늘도 승리!</div>
      </section>
      <footer class="todo-debug-statusbar"><span id="todo-status">LOCAL · READY</span><button id="todo-show-timer" type="button">TIMER</button></footer>
    </main>`;

  const list = container.querySelector<HTMLElement>("#todo-list")!;
  const summary = container.querySelector<HTMLElement>("#todo-summary")!;
  const current = container.querySelector<HTMLElement>("#todo-current")!;
  const form = container.querySelector<HTMLFormElement>(".todo-quick-add")!;
  const input = container.querySelector<HTMLInputElement>("#todo-input")!;
  const toggle = container.querySelector<HTMLButtonElement>("#todo-toggle-completed")!;
  const result = container.querySelector<HTMLElement>("#todo-focus-result")!;
  const resultTitle = container.querySelector<HTMLElement>("#todo-focus-result-title")!;
  const celebration = container.querySelector<HTMLElement>("#todo-celebration")!;
  const status = container.querySelector<HTMLElement>("#todo-status")!;
  let state: TodoSnapshot = { schemaVersion: 1, items: [], selectedTodoId: null, allCompletedCelebrated: false, activeFocusTodo: null, pendingFocusTodo: null };
  let completedHidden = false;
  let itemOrder: string[] = [];
  let deferSort = false;
  let sortTimer: number | undefined;
  let busy = false;

  const orderedIds = (): string[] => [...state.items.filter((item) => !item.completedAt), ...state.items.filter((item) => item.completedAt)].map((item) => item.id);
  const render = (): void => {
    const known = new Set(state.items.map((item) => item.id));
    itemOrder = itemOrder.filter((id) => known.has(id));
    for (const item of state.items) if (!itemOrder.includes(item.id)) itemOrder.push(item.id);
    if (!deferSort) itemOrder = orderedIds();
    const done = state.items.filter((item) => item.completedAt).length;
    summary.textContent = `${done} / ${state.items.length}`;
    const selected = state.items.find((item) => item.id === state.selectedTodoId);
    current.textContent = state.activeFocusTodo?.titleAtStart ?? selected?.text ?? "선택된 할 일 없음";
    toggle.textContent = completedHidden ? "완료 보기" : "완료 숨기기";
    list.replaceChildren();
    for (const id of itemOrder) {
      const item = state.items.find((candidate) => candidate.id === id);
      if (!item || (completedHidden && item.completedAt)) continue;
      const row = document.createElement("article");
      row.className = "todo-app-row";
      row.classList.toggle("completed", Boolean(item.completedAt));
      row.classList.toggle("selected", item.id === state.selectedTodoId);
      row.dataset.todoId = item.id;
      row.innerHTML = `<input type="checkbox" data-complete aria-label="완료"><button class="todo-item-text" data-select type="button"></button><button class="todo-more" data-edit type="button">수정</button><button class="todo-more danger" data-delete type="button">삭제</button>`;
      row.querySelector<HTMLInputElement>("[data-complete]")!.checked = Boolean(item.completedAt);
      const text = row.querySelector<HTMLButtonElement>("[data-select]")!;
      text.textContent = item.text;
      text.title = item.completedAt ? item.text : `${item.text} · 눌러서 집중 항목 선택`;
      text.disabled = Boolean(item.completedAt);
      list.append(row);
    }
    if (state.items.length === 0) list.innerHTML = `<div class="todo-app-empty"><b>오늘 할 일을 하나 적어보자!</b><span>작게 시작해도 충분해요.</span></div>`;
    result.hidden = !state.pendingFocusTodo;
    resultTitle.textContent = state.pendingFocusTodo?.titleAtStart ?? "";
  };

  const apply = (snapshot: TodoSnapshot): void => { state = snapshot; render(); };
  const run = async (command: string, payload: Record<string, unknown> = {}): Promise<void> => {
    if (busy) return;
    busy = true; status.textContent = "저장 중…";
    try { apply(await invoke<TodoSnapshot>(command, payload)); status.textContent = "저장됨"; }
    catch { status.textContent = "저장하지 못했습니다"; }
    finally { busy = false; }
  };

  try { apply(await invokeWhenReady<TodoSnapshot>("get_todo_state")); }
  catch { status.textContent = "할 일을 불러오지 못했습니다"; }
  form.addEventListener("submit", (event) => { event.preventDefault(); const text = input.value.trim(); if (!text) return; input.value = ""; void run("add_todo", { text }); input.focus(); });
  toggle.addEventListener("click", () => { completedHidden = !completedHidden; render(); });
  list.addEventListener("click", (event) => {
    const row = (event.target as Element).closest<HTMLElement>("[data-todo-id]"); if (!row) return;
    const item = state.items.find((candidate) => candidate.id === row.dataset.todoId); if (!item) return;
    if ((event.target as Element).closest("[data-select]")) void run("select_todo", { id: state.selectedTodoId === item.id ? null : item.id });
    else if ((event.target as Element).closest("[data-edit]")) { const text = window.prompt("할 일 수정", item.text)?.trim(); if (text) void run("update_todo", { id: item.id, text }); }
    else if ((event.target as Element).closest("[data-delete]")) { const linked = state.activeFocusTodo?.todoId === item.id; if (window.confirm(linked ? "집중 중인 할 일입니다. 연결을 해제하고 삭제할까요?" : "이 할 일을 삭제할까요?")) void run("delete_todo", { id: item.id }); }
  });
  list.addEventListener("change", (event) => {
    const check = (event.target as Element).closest<HTMLInputElement>("[data-complete]");
    const row = check?.closest<HTMLElement>("[data-todo-id]"); if (!check || !row) return;
    deferSort = true; window.clearTimeout(sortTimer); sortTimer = window.setTimeout(() => { deferSort = false; render(); }, 500);
    void run("set_todo_completed", { id: row.dataset.todoId, completed: check.checked });
  });
  result.addEventListener("click", (event) => { const button = (event.target as Element).closest<HTMLButtonElement>("[data-focus-action]"); if (button) void run("resolve_focus_todo", { action: button.dataset.focusAction }); });
  container.querySelector("#todo-start-focus")?.addEventListener("click", async () => { await invoke("start_focus"); await invoke("show_utility_window", { label: "timer" }); });
  container.querySelector("#todo-show-timer")?.addEventListener("click", () => void invoke("show_utility_window", { label: "timer" }));
  const unlistenChanged = await listen<TodoSnapshot>("todo://changed", ({ payload }) => apply(payload));
  const unlistenCompleted = await listen<TodoSnapshot>("todo://all-completed", ({ payload }) => { apply(payload); celebration.hidden = false; window.setTimeout(() => { celebration.hidden = true; }, 4500); });
  input.focus();
  return () => { window.clearTimeout(sortTimer); unlistenChanged(); unlistenCompleted(); };
}
