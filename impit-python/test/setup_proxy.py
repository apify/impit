import threading
import pproxy
import asyncio

def start_proxy_server(port: int = 3002):
    def run_proxy_server(stop_event):
        server = pproxy.Server(f'http://0.0.0.0:{port}')
        args = dict(rserver=[],
                    verbose=print)

        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        handler = loop.run_until_complete(server.start_server(args))

        try:
            while not stop_event.is_set():
                loop.run_until_complete(asyncio.sleep(0.1))
        except KeyboardInterrupt:
            print('exit!')

        handler.close()
        loop.run_until_complete(handler.wait_closed())
        loop.run_until_complete(loop.shutdown_asyncgens())
        loop.close()

    # Event to signal the proxy server to stop
    stop_event = threading.Event()

    # Start the proxy server in a separate thread
    proxy_thread = threading.Thread(target=run_proxy_server, args=(stop_event,), daemon=True)
    proxy_thread.start()

    # wait a moment for the server to start
    import time
    time.sleep(1)

    # Return a function to stop the server
    def stop_server():
        stop_event.set()
        proxy_thread.join(1)

    return stop_server
