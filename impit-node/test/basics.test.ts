import { test } from 'vitest';

import { HttpMethod, Impit } from '../index.js'
const impit = new Impit();

test('impit works', async (t) => {
  const response = await impit.fetch('https://jindrich.bar');

  const text = await response.text();
  
  t.expect(text).toContain('barjin');
})

test('json method works', async (t) => {
  const response = await impit.fetch('https://httpbin.org/json');

  const json = await response.json();

  t.expect(json?.slideshow?.author).toBe('Yours Truly');
})

test('headers work', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/headers', 
    { 
      headers: { 
        'Impit-Test': 'foo',
        'Cookie': 'test=123; test2=456'
      } 
    }
  );
  const json = await response.json();

  t.expect(json.headers?.['Impit-Test']).toBe('foo');
})

test('string request body works', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/post', 
    { 
      method: HttpMethod.Post,
      body: '{"Impit-Test":"foořžš"}',
      headers: { 'Content-Type': 'application/json' }
    }
  );
  const json = await response.json();

  t.expect(json.data).toEqual('{"Impit-Test":"foořžš"}');
});

test('binary request body works', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/post', 
    { 
      method: HttpMethod.Post,
      body: Uint8Array.from([0x49, 0x6d, 0x70, 0x69, 0x74, 0x2d, 0x54, 0x65, 0x73, 0x74, 0x3a, 0x66, 0x6f, 0x6f, 0xc5, 0x99, 0xc5, 0xbe, 0xc5, 0xa1]),
      headers: { 'Content-Type': 'application/json' }
    }
  );
  const json = await response.json();

  t.expect(json.data).toEqual('Impit-Test:foořžš');
});

test('redirects work by default', async (t) => {
  const response = await impit.fetch(
    'https://httpbin.org/absolute-redirect/1', 
  );

  t.expect(response.status).toBe(200);
});

test('disabling redirects works', async (t) => {
  const impit = new Impit({
    followRedirects: false
  });

  const response = await impit.fetch(
    'https://httpbin.org/absolute-redirect/1', 
  );

  t.expect(response.status).toBe(302);
  t.expect(response.headers['location']).toBe('http://httpbin.org/get');
});

test('limiting redirects works', async (t) => {
  const impit = new Impit({
    followRedirects: true,
    maxRedirects: 1
  });

  const response = impit.fetch(
    'https://httpbin.org/absolute-redirect/2', 
  );

  t.expect(response).rejects.toThrowError('TooManyRedirects');
});
