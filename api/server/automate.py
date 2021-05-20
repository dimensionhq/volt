import re
import json
import requests

generated = {}

name = input('Name: ')

response = requests.get(f'http://registry.npmjs.com/{name}').json()
version = response['dist-tags']['latest']
parent_version = version

generated['version'] = version
generated[version] = {}
generated[parent_version]['packages'] = {name: {}}

generated[parent_version]['packages'][name]['name'] = name
generated[parent_version]['packages'][name]['version'] = version
generated[parent_version]['packages'][name]['tarball'] = response['versions'][version]['dist']['tarball']
generated[parent_version]['packages'][name]['sha1'] = response['versions'][version]['dist']['shasum']
try:
    generated[parent_version]['packages'][name]['dependencies'] = list(
        response['versions'][version]['dependencies'].keys())
except KeyError:
    with open(rf'C:\Users\xtrem\Desktop\volt\api\server\public\{name}.json', 'w+') as f:
        f.write(json.dumps(generated))
    exit()


def add_dependencies_recursive(package_name: str, generated: dict, version: str):
    response = requests.get(f'http://registry.npmjs.com/{package_name}').json()
    generated[parent_version]['packages'][package_name] = {}

    if version == "*":
        version = list(response['versions'].keys())[-1]

    if '~' in version:
        # install latest patch version
        versions = list(response['versions'].keys())
        patch_version = '0'
        for pv in versions:
            opv = pv
            pv = pv.split('.')[:2]
            pv[0] = '~' + pv[0]

            if version.split('.')[:2] == pv and int(opv.split('.')[-1]) > int(
                patch_version[0]
            ):
                patch_version = opv[-1]

        version = '.'.join(version.split('.')[:2]) + '.' + patch_version
        version = version.replace('~', '')

    if '^' in version:
        # latest minor and patch version
        versions = list(response['versions'].keys())
        minor_version = '0'
        patch_version = '0'

        skip = False
        for v in versions:
            if not re.findall(r'[a-z]', v) and versions[-1] == version.replace('^', ''):
                version = version.replace('^', '')
                skip = True
                break

            if not re.findall(r'[a-z]', v) and v[0] == version[1]:
                split = v.split('.')[1]
                if not re.findall(r'[a-z]', v) and int(split) > int(minor_version):
                    minor_version = split
                    patch_version = '0'

                split2 = v.split('.')[2]

                if not re.findall(r'[a-z]', v) and int(split2) > int(patch_version):
                    patch_version = split2

        if not skip:
            major_version = version[1]
            version = major_version + '.' + minor_version + '.' + patch_version

    generated[parent_version]['packages'][package_name]['name'] = package_name
    generated[parent_version]['packages'][package_name]['version'] = version
    generated[parent_version]['packages'][package_name]['tarball'] = response['versions'][version]['dist']['tarball']
    generated[parent_version]['packages'][package_name]['sha1'] = response['versions'][version]['dist']['shasum']
    generated[parent_version]['packages'][package_name]['dependencies'] = list(
        response['versions'][version]['dependencies'].keys())

    if 'bin' in list(response['versions'][version].keys()):
        generated[parent_version]['packages'][package_name]['bin'] = response['versions'][version]['bin']

    try:
        for dependency, v in response['versions'][version]['dependencies'].items():
            v = v.replace('~', '').replace('^', '')
            add_dependencies_recursive(dependency, generated, v)
    except KeyError:
        pass


for dependency, version in response['versions'][version]['dependencies'].items():

    response = requests.get(f'http://registry.npmjs.com/{dependency}').json()
    generated[parent_version]['packages'][dependency] = {}

    if version == "*":
        version = list(response['versions'].keys())[-1]

    if '~' in version:
        # install latest patch version
        versions = list(response['versions'].keys())
        patch_version = '0'
        for pv in versions:
            opv = pv
            pv = pv.split('.')[:2]
            pv[0] = '~' + pv[0]

            if version.split('.')[:2] == pv and int(opv.split('.')[-1]) > int(
                patch_version[0]
            ):
                patch_version = opv[-1]

        version = '.'.join(version.split('.')[:2]) + '.' + patch_version
        version = version.replace('~', '')

    if '^' in version:
        # latest minor and patch version
        versions = list(response['versions'].keys())
        minor_version = '0'
        patch_version = '0'

        skip = False
        for v in versions:
            if not re.findall(r'[a-z]', v) and versions[-1] == version.replace('^', ''):
                version = version.replace('^', '')
                skip = True
                break

            if not re.findall(r'[a-z]', v) and v[0] == version[1]:
                split = v.split('.')[1]
                if not re.findall(r'[a-z]', v) and int(split) > int(minor_version):
                    minor_version = split
                    patch_version = '0'

                split2 = v.split('.')[2]

                if not re.findall(r'[a-z]', v) and int(split2) > int(patch_version):
                    patch_version = split2

        if not skip:
            major_version = version[1]
            version = major_version + '.' + minor_version + '.' + patch_version

    generated[parent_version]['packages'][dependency]['name'] = dependency
    generated[parent_version]['packages'][dependency]['version'] = version
    generated[parent_version]['packages'][dependency]['tarball'] = response['versions'][version]['dist']['tarball']
    generated[parent_version]['packages'][dependency]['sha1'] = response['versions'][version]['dist']['shasum']

    try:
        generated[parent_version]['packages'][dependency]['dependencies'] = list(
            response['versions'][version]['dependencies'].keys())
    except:
        generated[parent_version]['packages'][dependency]['dependencies'] = []

    if 'bin' in list(response['versions'][version].keys()):
        generated[parent_version]['packages'][dependency]['bin'] = response['versions'][version]['bin']

    try:
        for dependency, v in response['versions'][version]['dependencies'].items():
            v = v.replace('~', '').replace('^', '')
            add_dependencies_recursive(dependency, generated, v)
    except KeyError:
        pass

with open(rf'C:\Users\xtrem\Desktop\volt\api\server\public\{name}.json', 'w+') as f:
    f.write(json.dumps(generated))
