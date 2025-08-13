import { getRandomBytes as _ } from 'wasi:random/random@0.2.6';
import { getDirectories } from 'wasi:filesystem/preopens@0.2.6';

const customEndpoint = {
  handleRequest: async function(input) {
    try {
      const addr = 'http://google.com';
      const resp = await fetch(addr);
      const text = await resp.text();

      const dirs = getDirectories();

      console.log(`Hello from JS guest [${input}]: /get(${addr}) => ${text}\n${dirs}`);
    } catch (err) {
      console.error(`Error: ${err}`);
    }

    return input;
  }
};

export {
  customEndpoint,
};
