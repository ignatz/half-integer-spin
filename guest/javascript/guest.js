import { listDirectories } from "./runtime.js";

import {
  ResponseOutparam,
  OutgoingBody,
  OutgoingResponse,
  Fields,
} from 'wasi:http/types@0.2.3';

export const incomingHandler = {
  handle: async function(incomingRequest, responseOutparam) {
    const outgoingResponse = new OutgoingResponse(new Fields());

    // Access the outgoing response body
    let outgoingBody = outgoingResponse.body();
    {
      // Create a stream for the response body
      let outputStream = outgoingBody.write();
      outputStream.blockingWriteAndFlush(
        new Uint8Array(new TextEncoder().encode('Hello from Javascript!\n'))
      );
      outputStream[Symbol.dispose]();
    }

    outgoingResponse.setStatusCode(200);
    OutgoingBody.finish(outgoingBody, undefined);
    ResponseOutparam.set(responseOutparam, { tag: 'ok', val: outgoingResponse });
  }
};

// addEventListener("fetch", (event) =>
//   event.respondWith(
//     (async () => {
//       return new Response("Hello World");
//     })(),
//   ),
// );

let globalState = 1;
const _other = (() => {
  globalState = 5;
  return 23;
})();

let handlers = {}

function addCallback(key, f) {
  handlers[key] = f;
}

addCallback("test0", () => 4);
addCallback("test1", () => 4);

export const customEndpoint = {
  handleRequest: async function(input) {
    try {
      const addr = 'http://google.com';
      const resp = await fetch(addr);
      const text = await resp.text();

      const dirs = listDirectories();

      globalState++;
      console.log(`Hello ${globalState} ${Object.keys(handlers)} from JS guest [${input}]: /get(${addr}) => ${text}\n${dirs}`);
    } catch (err) {
      console.error(`Error: ${err}`);
    }

    return input;
  }
};
