import init, { run_app } from './pkg/format_galaxy_web_host.js';
async function main() {
   await init('./pkg/format_galaxy_web_host_bg.wasm');
   run_app();
}
main()