import subprocess
import sys
import textwrap

# The result delivery of the last awaited call releases the GIL inside `call_soon_threadsafe`
# (`socket.send`). If the interpreter starts finalizing before the delivering tokio thread gets the
# GIL back, CPython < 3.13.8 kills that thread with `pthread_exit`, which crashes the process.
# The script below widens both windows so the race is hit on every run.
SCRIPT = textwrap.dedent("""
    import asyncio
    import asyncio.selector_events as selector_events
    import sys
    import time
    import types

    import impit


    class SlowTeardown:
        def __del__(self, _now=time.perf_counter):
            end = _now() + 0.1
            while _now() < end:
                pass


    # Torn down after the interpreter is marked as finalizing.
    module = types.ModuleType('_slow_teardown')
    module.obj = SlowTeardown()
    sys.modules[module.__name__] = module
    del module


    def _write_to_self(self, _orig=selector_events.BaseSelectorEventLoop._write_to_self, _sleep=time.sleep):
        _orig(self)
        _sleep(0.02)


    selector_events.BaseSelectorEventLoop._write_to_self = _write_to_self


    async def main():
        async with impit.AsyncClient():
            pass


    asyncio.run(main())
""")


def test_exit_right_after_await_does_not_crash() -> None:
    result = subprocess.run([sys.executable, '-c', SCRIPT], capture_output=True, timeout=30, check=False)  # noqa: S603
    assert result.returncode == 0, result.stderr.decode()


# atexit hooks run in reverse order, so this one runs after the hook impit registers on import.
LATE_ATEXIT_SCRIPT = textwrap.dedent("""
    import asyncio
    import atexit


    async def main():
        async with impit.AsyncClient():
            pass


    atexit.register(lambda: asyncio.run(main()))

    import impit
""")


def test_async_client_works_in_atexit_hook_registered_before_import() -> None:
    result = subprocess.run([sys.executable, '-c', LATE_ATEXIT_SCRIPT], capture_output=True, timeout=10, check=False)  # noqa: S603
    assert result.returncode == 0, result.stderr.decode()
