export * from "../voxel_rsmcdoc.js";
import init, { McDocValidator as WasmValidator } from '../voxel_rsmcdoc.js';

// Variable pour tracking l'état d'initialisation
let wasmInitialized = false;
let initPromise: Promise<void> | null = null;

/**
 * Initialise le module WASM - appelé automatiquement lors du premier usage
 * @param wasmPath - Chemin optionnel vers le fichier WASM
 */
export async function initWasm(wasmPath?: string | URL): Promise<void> {
  if (wasmInitialized) return;
  
  if (!initPromise) {
    initPromise = (async () => {
      await init(wasmPath);
      wasmInitialized = true;
    })();
  }
  
  return initPromise;
}

/**
 * Classe McDocValidator avec auto-initialisation
 */
export class McDocValidator {
  private wasmInstance: WasmValidator | null = null;

  async init(): Promise<void> {
    if (this.wasmInstance) return;
    
    await initWasm();
    this.wasmInstance = new WasmValidator();
  }

  async load_mcdoc_files(files: any): Promise<void> {
    await this.init();
    return this.wasmInstance!.load_mcdoc_files(files);
  }

  async load_registries(registries: any, version: string): Promise<void> {
    await this.init();
    return this.wasmInstance!.load_registries(registries, version);
  }

  async validate_json(json: any, resource_type: string): Promise<any> {
    await this.init();
    return this.wasmInstance!.validate_json(json, resource_type);
  }

  async get_required_registries(json: any, resource_type: string): Promise<any> {
    await this.init();
    return this.wasmInstance!.get_required_registries(json, resource_type);
  }

  async analyze_datapack(files: any): Promise<any> {
    await this.init();
    return this.wasmInstance!.analyze_datapack(files);
  }

  free(): void {
    this.wasmInstance?.free();
    this.wasmInstance = null;
  }
}
