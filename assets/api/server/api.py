from flask import Flask, jsonify
import json

app = Flask(__name__)


@app.route('/')
def index():
    return 'No file queried.'


@app.route('/<string:package>')
def get(package: str):
    data = ''
    with open(f'public/{package}.json') as f:
        data = f.read()

    return jsonify(json.loads(data))


if __name__ == "__main__":
    app.run()
