import init, { DatapackValidator as WasmValidator } from '../voxel_rsmcdoc.js';

/**
 * DatapackValidator avec API moderne et auto-initialisation
 */
export class DatapackValidator {
  private constructor(private wasm: WasmValidator) { }

  /**
   * Factory method - initialise automatiquement WASM et validator
   */
  static async init(
    registries: Record<string, any>,
    mcdocFiles: Record<string, string>,
    version: string
  ): Promise<DatapackValidator> {
    await init();
    const wasm = WasmValidator.init(registries, mcdocFiles, version);
    return new DatapackValidator(wasm);
  }

  /**
   * Valide un JSON contre un type de ressource
   */
  validate(json: any, resourceType: string, version?: string) {
    return this.wasm.validate(json, resourceType, version);
  }

  /**
   * Analyse un datapack complet
   */
  analyzeDatapack(files: Record<string, any>) {
    return this.wasm.analyze_datapack(files);
  }
}

// Re-exports
export * from "../voxel_rsmcdoc.js";
export { DatapackValidator as McDocValidator };
