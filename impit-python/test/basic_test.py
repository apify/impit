from impit import Client

def test_basic_usage():
    client = Client()
    assert "origin" in client.get("https://httpbin.org/get").text

