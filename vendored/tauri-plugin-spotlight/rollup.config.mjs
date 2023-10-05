import { readFileSync } from 'fs'
import { builtinModules } from 'module'
import typescript from '@rollup/plugin-typescript'
import resolve from '@rollup/plugin-node-resolve'
import terser from '@rollup/plugin-terser'

/**
 * Create a base rollup config
 * @param {Record<string,any>} pkg Imported package.json
 * @param {string[]} external Imported package.json
 * @returns {import('rollup').RollupOptions}
 */
export function createConfig ({ input = 'index.ts', pkg, external = [] }) {
  return [
    {
      input,
      external: Object.keys(pkg.dependencies || {})
        .concat(Object.keys(pkg.peerDependencies || {}))
        .concat(builtinModules)
        .concat(external),
      onwarn: (warning) => {
        throw Object.assign(new Error(), warning)
      },
      strictDeprecations: true,
      output: {
        file: pkg.module,
        format: 'es',
        sourcemap: true,
      },
      plugins: [
        terser(),
        typescript({ sourceMap: true, declaration: true, outDir: 'dist-js' }),
      ],
    },
    {
      input,
      onwarn: (warning) => {
        throw Object.assign(new Error(), warning)
      },
      strictDeprecations: true,
      output: {
        file: pkg.browser,
        format: 'es',
        sourcemap: true,
        entryFileNames: '[name].min.js',
      },
      plugins: [
        resolve(),
        terser(),
        typescript({ sourceMap: true, declaration: true, outDir: 'dist-js' }),
      ],
    },
  ]
}

export default createConfig({
  input: 'guest-js/index.ts',
  pkg: JSON.parse(
    readFileSync(new URL('./package.json', import.meta.url), 'utf8'),
  ),
  external: [/^@tauri-apps\/api/],
})
