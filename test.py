from timeit import default_timer as timer
import requests
import json


start = timer()
response = requests.get('http://registry.yarnpkg.com/react')
data = json.loads(response.text)
end = timer()

print(end - start)