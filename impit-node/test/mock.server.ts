import express from 'express';
import { Server } from 'http';

export const routes = {
    charset: {
        path: '/charset',
        bodyBuffer: Buffer.from([0x50, 0xf8, 0xed, 0x6c, 0x69, 0x9a, 0x20, 0x9e, 0x6c, 0x75, 0x9d, 0x6f, 0x75, 0xe8, 0x6b, 0xfd, 0x20, 0x6b, 0xf9, 0xf2, 0x20, 0xfa, 0x70, 0xec, 0x6c, 0x20, 0xef, 0xe1, 0x62, 0x65, 0x6c, 0x73, 0x6b, 0xe9, 0x20, 0xf3, 0x64, 0x79]),
        bodyString: 'Příliš žluťoučký kůň úpěl ďábelské ódy'
    }
}

export async function runServer(port: number): Promise<Server> {
    const app = express();

    app.get(routes.charset.path, (req, res) => {
        res.set('Content-Type', 'text/plain; charset=windows-1250');
        res.send(routes.charset.bodyBuffer);
    });

    return new Promise((res,rej) => {
        const server = app.listen(port, (err) => {
            if (err) {
                rej(err);
            } else {
                res(server);
            }
        });
    });
}
