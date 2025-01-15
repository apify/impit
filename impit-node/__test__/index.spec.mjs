import test from 'ava'

import { Impit } from '../index.js'
const impit = new Impit();

test('impit works', async (t) => {
  const response = await impit.fetch('https://jindrich.bar');

  const text = await response.text();
  text.includes('barjin') ? t.pass() : t.fail();
})

test('json method works', async (t) => {
  const response = await impit.fetch('https://httpbin.org/json');

  const json = await response.json();
  json.slideshow.author === 'Yours Truly' ? t.pass() : t.fail();
})

test('headers work', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/headers', 
    { 
      headers: { 
        'Retch-Test': 'foo',
        'Cookie': 'test=123; test2=456'
      } 
    }
  );
  const json = await response.json();

  json.headers['Retch-Test'] ? t.pass() : t.fail();
})

test('string request body works', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/post', 
    { 
      method: 'POST',
      body: '{"Retch-Test":"foořžš"}',
      headers: { 'Content-Type': 'application/json' }
    }
  );
  const json = await response.json();

  json.data === '{"Retch-Test":"foořžš"}' ? t.pass() : t.fail();
});

test('binary request body works', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/post', 
    { 
      method: 'POST',
      body: Uint8Array.from([0x52, 0x65, 0x74, 0x63, 0x68, 0x2d, 0x54, 0x65, 0x73, 0x74, 0x3a, 0x66, 0x6f, 0x6f, 0xc5, 0x99, 0xc5, 0xbe, 0xc5, 0xa1]),
      headers: { 'Content-Type': 'application/json' }
    }
  );
  const json = await response.json();

  t.deepEqual(json.data, 'Retch-Test:foořžš');
});

test('redirects work by default', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/absolute-redirect/1', 
  );

  t.deepEqual(response.status, 200);
});

test('disabling redirects works', async (t) => {
  const impit = new Impit({
    followRedirects: false
  });

  const response = await impit.fetch(
    'https://httpbin.org/absolute-redirect/1', 
  );

  t.deepEqual(response.status, 302);
  t.deepEqual(response.headers['location'], 'http://httpbin.org/get');
});

test('limiting redirects works', async (t) => {
  const impit = new Impit({
    followRedirects: true,
    maxRedirects: 1
  });

  const response = impit.fetch(
    'https://httpbin.org/absolute-redirect/2', 
  );

  await t.throwsAsync(response, { message: /TooManyRedirects/ });
});
