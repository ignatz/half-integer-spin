import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, parse } from 'node:path';

import { componentize } from '@bytecodealliance/componentize-js';

// AoT compilation makes use of weval (https://github.com/bytecodealliance/weval)
const enableAot = process.env.ENABLE_AOT == '1';

const filename = 'guest.js';
const base = parse(filename).name;
const wit = `../../wit/`;

console.log(`compiling (${filename}, ${wit}) with AoT = ${enableAot}`);

const { component } = await componentize(await readFile(filename, 'utf8'), {
  witPath: resolve(wit),
  enableAot
});

const targetDir = 'dist';
await mkdir(targetDir, { recursive: true });
await writeFile(`${targetDir}/${base}.component.wasm`, component);
