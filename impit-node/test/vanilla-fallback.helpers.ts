export function createTransportFailure(): Error {
    return new Error(
        'HTTPError: The internal HTTP library has thrown an error:\n'
        + 'reqwest::Error { kind: Request, source: hyper_util::client::legacy::Error('
        + 'SendRequest, hyper::Error(Io, Custom { kind: ConnectionReset, error: "Connection reset by peer" })) }',
    );
}

export function createWrappedResponse(body: string, url: string): Response {
    const bytes = Buffer.from(body);
    const response = new Response(bytes, {
        status: 200,
        headers: { 'Content-Type': 'text/plain' },
    });

    Object.defineProperty(response, 'bytes', {
        value: async () => new Uint8Array(bytes),
        configurable: true,
    });
    Object.defineProperty(response, 'decodeBuffer', {
        value: (buffer: ArrayBuffer | ArrayBufferView) => Buffer.from(buffer as any).toString('utf8'),
        configurable: true,
    });
    Object.defineProperty(response, 'abort', {
        value: () => {},
        configurable: true,
    });
    Object.defineProperty(response, 'url', {
        value: url,
        configurable: true,
    });

    return response;
}

export const LOCALHOST_TLS_KEY = `-----BEGIN PRIVATE KEY-----
MIIEwAIBADANBgkqhkiG9w0BAQEFAASCBKowggSmAgEAAoIBAQDMxR6FpXcONRoz
9XgftbFKMJCuoGsa0eIooASG/PIUzkl08UhqM+60UgjwQX/HnA/ElXqBamnmXyK0
drO6t+RoatX4YCkJ4E0EVjOQgoZe1R6/tld1MT69OUkndPm0D4amTVYrW8xuBQk/
y3DOiBNVkRc4VaMgdHd3VnnFH+HZFHdADyh3kchQ/vxj4MarCTcONZIjzrkqoK1y
c6Hi2gCFN3gGypVVerNMKh24qYRwqVknKiMzRnCudqwQeeFysyHwYMujVw7/PAJS
fV9s1G3Ui3bQmQcGivFELC2SK7dGcr5tK5MBioVLbVkHN8hAwUSZiOJAAEKItzPl
cpGoPSOJAgMBAAECggEBAME+ZYeKl8h4pLnUNgD23tE8881Y5rrwx5W/LYaWv36T
Dw+lhMl1KRhTMsxJg+VEijzjNDFd04Ls1TupqgPT92HzMOqtFQ2U+BnXn+IIy/ZC
+jnCQtb+Gk9I+Jib8+rRnCjlYySYBVzus8PYoiTGljhyLI+lgcTnJLcijNhTNjg9
P4YkUK8OqNs9TMjs7vb7dDDsBi+hBYVlpnt6Bnc/rh8BxL2z6ikHXYvitMKHozzj
rVvLZyMbfC1sdHAyEz2YpHgcW2+ePRqc/+/Yy5BmutZSGrxpGMH9Uo0hhNtjkbe7
gzcl/6ztR7eozap2mXZllg/Vg8MFvFhAY7zK6YOH3AECgYEA8FFRp29/iGzqBwOH
5mmo0wgJH5DbtiM3L7Yqk12/sS86NQp87KMcNiCPRYs/8M0NfLr61vdNI9zBL0IU
kBgiTfYXY8oNPGhjqSC9qEpPxuM0qHpv9BpYhRdegWs1gqHOoi68HuoAaDnSS73D
skop8cxwzYDcjRUQxxnY8bNG6BkCgYEA2iHzauXVr3myLla8tqYQggo2axu+gXZt
pbtbKWccE2fv7z5tnS+ZChZxv3x4FvLBIzAuF/RONLV15uph/gIz+Crk01GMqLr2
kgexGSVKSvc+9b9phj51bOSy16SXpUgIw1LDEd2JkIPP0Avxs7Hq8xS0ITpqJMmX
zjGP3QZGRPECgYEAjiEKEeS3oJAJuSw1a+iBmI3gF3Ms/oPFV8p9U7rWbIxp+ITD
bZDqVnjbQ14f6uLbXzGWuRx52wPsnW6Pisk7QLCTFMmjGl8C0jwy7x1EIXSu6BXB
sLUENXKkyhYGB8R62SCa0g3DP+EypukMnJ2QQRmQfXoA9s/GpHp8/DXzccECgYEA
nM6bNdVS73oEZNtlfceTRmghBo5DPL3txJ4Swoik3i5xhQLTuZNl6KKJ0qWfjp+j
x6/y8rVlIu7vergzCW57/YKYTHDrNMByUDfHT9RGu+1RDUg0i5SKxWUCS5K+kMpf
wknUgRtIsOKQmXZ8ojjcNTJE6z4a36crwcZPLQw9p4ECgYEAtCGgy3prNRaU4Ykh
SSBsz64u4hrMWdHG+UdzVabnFpcKQAGrqkX9bZvYe0yx9G0vBv/TiDTVPZcGXxT+
ghz/U85A1Z/epyvSJzhzMTK9VxrH9pTECNTJTz5FOp2FlU1Eqv1GpJedBtH57+fg
4t42YFuPOLNLfQCsHLCUJsTPMnI=
-----END PRIVATE KEY-----`;

export const LOCALHOST_TLS_CERT = `-----BEGIN CERTIFICATE-----
MIICpDCCAYwCCQCRYoVtQ3nVXjANBgkqhkiG9w0BAQsFADAUMRIwEAYDVQQDDAls
b2NhbGhvc3QwHhcNMjYwNDI5MDA0ODEzWhcNMjYwNDMwMDA0ODEzWjAUMRIwEAYD
VQQDDAlsb2NhbGhvc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDM
xR6FpXcONRoz9XgftbFKMJCuoGsa0eIooASG/PIUzkl08UhqM+60UgjwQX/HnA/E
lXqBamnmXyK0drO6t+RoatX4YCkJ4E0EVjOQgoZe1R6/tld1MT69OUkndPm0D4am
TVYrW8xuBQk/y3DOiBNVkRc4VaMgdHd3VnnFH+HZFHdADyh3kchQ/vxj4MarCTcO
NZIjzrkqoK1yc6Hi2gCFN3gGypVVerNMKh24qYRwqVknKiMzRnCudqwQeeFysyHw
YMujVw7/PAJSfV9s1G3Ui3bQmQcGivFELC2SK7dGcr5tK5MBioVLbVkHN8hAwUSZ
iOJAAEKItzPlcpGoPSOJAgMBAAEwDQYJKoZIhvcNAQELBQADggEBAIG+juslOvfW
m3FxmsTuTS+ZCu+RLZPslXHvnHRNXbVmGcRfGdXJQNnhZyXkB9sfrlD5DRn5dwiE
4g5oH7qcdSZ86EsBU9zePFD1IBZa1jby3VAf+rGmwL8jjHjQp8Oc2++jdC/ecft+
EoX6Fj+g2v4F2cWn3YRBKYb2h4Fp02ejKktypRv1JUp3m6D2ar5591MKjBahYW8n
ULIQMxStpMK1b0GZa2WKhXhRNnNa21ehyfWNBQw9QI1OjedJItLXfOZBzXrT9WAC
RcYwHeuM43ak33DUO93y/AnaSQRxpdLNRW7q5L8uX5ly1XR9u5aPjq71yUj8cHKr
moxci9K2uno=
-----END CERTIFICATE-----`;
