export async function hello(name) {
  try {
    const resp = await fetch('http://google.com');
    const text = await resp.text();
    return `Aysnc hello response: ${text}`;
  } catch (err) {
    return `Error: ${err}`;
  }
  return `Aysnc hello ${name}`;
}
