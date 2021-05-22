import requests
import json
import re


def get_dependencies(data, version):
    try:
        return list(data['versions'][version]['dependencies'].keys())
    except KeyError:
        return []


def get_cleaned_version(unclean_version, data):
    version = ''

    if unclean_version == "*":
        version = list(data['versions'].keys())[-1]

    if '~' in unclean_version:
        # install latest patch version
        versions = list(data['versions'].keys())

        patch_version = '0'
        for pv in versions:
            opv = pv
            pv = pv.split('.')[:2]
            pv[0] = '~' + pv[0]

            if not re.findall(r'[a-z]', opv) and unclean_version.split('.')[:2] == pv and int(opv.split('.')[-1]) > int(
                patch_version[0]
            ):
                patch_version = opv[-1]

        version = '.'.join(unclean_version.split('.')
                           [:2]) + '.' + patch_version
        version = version.replace('~', '')

    if '^' in unclean_version:
        # latest minor and patch version
        versions = list(data['versions'].keys())
        minor_version = '0'
        patch_version = '0'

        skip = False
        for v in versions:
            if not re.findall(r'[a-z]', v) and versions[-1] == unclean_version.replace('^', ''):
                version = unclean_version.replace('^', '')
                skip = True
                break

            if not re.findall(r'[a-z]', v) and v[0] == unclean_version[1]:
                split = v.split('.')[1]
                if not re.findall(r'[a-z]', v) and int(split) > int(minor_version):
                    minor_version = split
                    patch_version = '0'

                split2 = v.split('.')[2]
                print(v)
                if not re.findall(r'[a-z]', v) and int(split2) > int(patch_version):
                    patch_version = split2

        if not skip:
            major_version = unclean_version[1]
            version = major_version + '.' + minor_version + '.' + patch_version

    version = version.replace('^', '').replace('~', '')

    return version if version != '' else unclean_version


def get_dependencies_recursive(generated, dependency, unclean_version, main_version):
    response = requests.get(
        rf'http://registry.yarnpkg.com/{dependency}').json()

    version = get_cleaned_version(unclean_version, response)

    print("checking ", dependency);

    print("version: ", version);

    dependencies = get_dependencies(response, version)

    print("dependencies: ", dependencies);

    version_split = version.split(' ');
    print("split:", version_split)
    if len(version_split) > 1:
        version = version_split[1]
    else:
        version = version_split[0]
    print("ver:", version)
    tarball = response['versions'][version]['dist']['tarball'];    

    generated[main_version]['packages'][dependency] = {
        'name': dependency,
        'version': version,
        'tarball': tarball,
        'sha1': response['versions'][version]['dist']['shasum'],
        'dependencies': dependencies,
    }

    if 'bin' in list(response['versions'][version].keys()):
        generated[main_version]['packages'][package]['bin'] = response['versions'][version]['bin']

    try:
        for dep, vers in response['versions'][version]['dependencies'].items():
            # print("dep:", dep)
            get_dependencies_recursive(generated, dep, vers, main_version)
    except KeyError:
        pass


generated = {}

package = input('Enter package name: ')


main_response = requests.get(rf'http://registry.yarnpkg.com/{package}').json()

main_version = main_response['dist-tags']['latest']
main_version = get_cleaned_version(main_version, main_response)

# Set Version

generated['version'] = main_version


generated[main_version] = {'packages': {}}

generated[main_version]['packages'][package] = {
    'name': package,
    'version': main_version,
    'tarball': main_response['versions'][main_version]['dist']['tarball'],
    'sha1': main_response['versions'][main_version]['dist']['shasum'],
    'dependencies': get_dependencies(main_response, main_version)
}

if 'bin' in list(main_response['versions'][main_version].keys()):
    generated[main_version]['packages'][package]['bin'] = main_response['versions'][main_version]['bin']

for main_dependency, version in main_response['versions'][main_version]['dependencies'].items():
    # print('Adding: ', main_dependency)
    get_dependencies_recursive(
        generated, main_dependency, version, main_version)

with open(rf'public\{package}.json', 'w+') as f:
    f.write(json.dumps(generated))
