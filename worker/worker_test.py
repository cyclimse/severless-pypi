from fastapi.testclient import TestClient

from worker import app

client = TestClient(app)


def test_read_main():
    response = client.post(
        "/",
        json={
            "project": "dagon",
            "archive_url": "https://test-files.pythonhosted.org/packages/ef/81/0265e52f586cfc8c295dbaae8befe397db6bf31f071373c5e2f14fb3f7f7/dagon-0.9.9.tar.gz",
        },
    )
    assert response.status_code == 200
    assert response.json() == {"msg": "Hello World"}
