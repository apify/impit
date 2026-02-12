// Taken from https://github.com/nodejs/undici/blob/14e62db0d0cff4bea27357aa5bd14881459b27c7/lib/web/fetch/body.js#L120
// patched for use with Impit
async function generateMultipartFormData(formData) {
    const boundary = `----formdata-impit-0${`${Math.random().toString().slice(0, 5)}`.padStart(11, '0')}`;
    const prefix = `--${boundary}\r\nContent-Disposition: form-data`;

    /*! formdata-polyfill. MIT License. Jimmy Wärting <https://jimmy.warting.se/opensource> */
    const escape = (str) => str.replace(/\n/g, '%0A').replace(/\r/g, '%0D').replace(/"/g, '%22');
    const normalizeLinefeeds = (value) => value.replace(/\r?\n|\r/g, '\r\n');

    // Set action to this step: run the multipart/form-data
    // encoding algorithm, with object’s entry list and UTF-8.
    // - This ensures that the body is immutable and can't be changed afterwords
    // - That the content-length is calculated in advance.
    // - And that all parts are pre-encoded and ready to be sent.
    const blobParts = [];
    const rn = new Uint8Array([13, 10]); // '\r\n'
    const textEncoder = new TextEncoder();

    for (const [name, value] of formData) {
        if (typeof value === 'string') {
            const chunk = textEncoder.encode(prefix +
                `; name="${escape(normalizeLinefeeds(name))}"` +
                `\r\n\r\n${normalizeLinefeeds(value)}\r\n`);
            blobParts.push(chunk);
        } else {
            const chunk = textEncoder.encode(`${prefix}; name="${escape(normalizeLinefeeds(name))}"` +
                (value.name ? `; filename="${escape(value.name)}"` : '') + '\r\n' +
                `Content-Type: ${value.type || 'application/octet-stream'}\r\n\r\n`);
            blobParts.push(chunk, value, rn);
        }
    }

    // CRLF is appended to the body to function with legacy servers and match other implementations.
    // https://github.com/curl/curl/blob/3434c6b46e682452973972e8313613dfa58cd690/lib/mime.c#L1029-L1030
    // https://github.com/form-data/form-data/issues/63
    const chunk = textEncoder.encode(`--${boundary}--\r\n`);
    blobParts.push(chunk);

    const action = async function* () {
        for (const part of blobParts) {
            if (part.stream) {
                yield* part.stream();
            } else {
                yield part;
            }
        }
    };

    const parts = [];
    for await (const part of action()) {
        if (part instanceof Uint8Array) {
            parts.push(part);
        } else if (part instanceof Blob) {
            const arrayBuffer = await part.arrayBuffer();
            parts.push(new Uint8Array(arrayBuffer));
        } else {
            throw new TypeError('Unsupported part type');
        }
    }
    const body = new Uint8Array(parts.reduce((acc, part) => acc + part.length, 0));
    let offset = 0;
    for (const part of parts) {
        body.set(part, offset);
        offset += part.length;
    }

    // Set type to `multipart/form-data; boundary=`,
    // followed by the multipart/form-data boundary string generated
    // by the multipart/form-data encoding algorithm.
    return {
        body,
        type: `multipart/form-data; boundary=${boundary}`,
    };
}


// logic from https://github.com/nodejs/undici/blob/14e62db0d0cff4bea27357aa5bd14881459b27c7/lib/web/fetch/body.js#L90
async function castToTypedArray(body) {
    let typedArray = body;
    let type = "";

    if (typeof body === 'string') {
        typedArray = new TextEncoder().encode(body);
        type = 'text/plain;charset=UTF-8';
    }
    else if (typedArray instanceof URLSearchParams) {
        typedArray = new TextEncoder().encode(body.toString());
        type = 'application/x-www-form-urlencoded;charset=UTF-8';
    } else if (body instanceof ArrayBuffer) {
        typedArray = new Uint8Array(body.slice());
    } else if (ArrayBuffer.isView(body)) {
        typedArray = new Uint8Array(body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength));
    } else if (body instanceof Blob) {
        typedArray = new Uint8Array(await body.arrayBuffer());
        type = body.type;
    } else if (body instanceof FormData) {
        return await generateMultipartFormData(body);
    } else if (body instanceof ReadableStream) {
        const reader = body.getReader();
        const chunks = [];

        let done = false;
        while (!done) {
            const { done: streamDone, value } = await reader.read();
            done = streamDone;
            if (value) {
                chunks.push(value);
            }
        }

        const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
        typedArray = new Uint8Array(totalLength);
        let offset = 0;

        for (const chunk of chunks) {
            typedArray.set(chunk, offset);
            offset += chunk.length;
        }
    }


    return { body: typedArray, type };
}

exports.castToTypedArray = castToTypedArray;
