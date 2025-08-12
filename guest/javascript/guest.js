const customEndpoint = {
  handleRequest: async function(input) {
    try {
      const addr = 'http://google.com';
      const resp = await fetch(addr);
      const text = await resp.text();

      console.log(`Hello from JS guest [${input}]: /get(${addr}) => ${text}`);
    } catch (err) {
      console.error(`Error: ${err}`);
    }

    return input;
  }
};

export {
  customEndpoint,
};
