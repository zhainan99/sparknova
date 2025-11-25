import { b as attr } from "../../chunks/attributes.js";
import "@tauri-apps/api/window";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let query = "";
    $$renderer2.push(`<div class="app-container svelte-1uha8ag"><div class="search-input-container svelte-1uha8ag"><input${attr("value", query)} type="text" class="search-input svelte-1uha8ag" placeholder="输入命令或搜索..."/></div></div>`);
  });
}
export {
  _page as default
};
