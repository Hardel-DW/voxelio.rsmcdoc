import { defineConfig } from 'tsdown'

export default defineConfig({
  entry: ['./src/index.ts'],
  platform: 'neutral',
  minify: true,
  target: 'es2020',
  copy: [
    {
      from: './voxel_rsmcdoc.js',
      to: './dist/voxel_rsmcdoc.js',
    },
    {
      from: './voxel_rsmcdoc.d.ts',
      to: './dist/voxel_rsmcdoc.d.ts',
    },
    {
      from: './voxel_rsmcdoc_bg.wasm',
      to: './dist/voxel_rsmcdoc_bg.wasm',
    },
    {
      from: './voxel_rsmcdoc_bg.wasm.d.ts',
      to: './dist/voxel_rsmcdoc_bg.wasm.d.ts',
    },
  ],
  dts: {
    isolatedDeclarations: true,
  },
})
