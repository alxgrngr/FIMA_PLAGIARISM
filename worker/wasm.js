import compiled from './pkg/receipt_core_bg.wasm';
import * as bindings from './pkg/receipt_core_bg.js';
const instance=new WebAssembly.Instance(compiled,{'./receipt_core_bg.js':bindings});
bindings.__wbg_set_wasm(instance.exports);
instance.exports.__wbindgen_start();
bindings.warm_model();
export { assess } from './pkg/receipt_core_bg.js';
