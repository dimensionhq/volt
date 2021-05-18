import json
import requests
# from dbmanager import DbManager
# from flask import Flask, jsonify

# app = Flask(__name__)

# dm = DbManager()
# db = dm.initialise()


# @app.route("/")
# def home():
#     return "A resource was not requested."


# @app.route('/<string:name>')
# def package(name: str):
#     return jsonify((DbManager.get(name, db)))


# if __name__ == "__main__":
#     app.run()


arr = []


def get_package(name: str):
    if '^' in name or '*' in name:
        name = name.split('^')[0].replace('*', '')[:-1]
    elif name.count('@') == 2:
        name = '@' + name.split('@')[1].replace('*', '')
    elif '^' not in name and '*' not in name and name.count('@') == 1:
        name = name.split('@')[0]

    data = requests.get(
        f'http://registry.npmjs.com/{name}').json()
    latest_version = data['dist-tags']['latest']
    try:
        dependencies = data['versions'][latest_version]['dependencies']
    except:
        dependencies = {}

    na = [f'{k}@{v}' for k, v in dependencies.items()]

    for val in na:
        if val not in arr:
            arr.append(val)
        get_package(val)


name = 'jquery'
version = '3.5.1'

get_package(f'{name}@^1.23')

doc = {
    'dependencies': {
        version: arr
    }
}

with open(fr'C:\Users\xtrem\Desktop\volt\api\server\public\{name}.json', 'w+') as f:
    f.write(json.dumps(doc))
