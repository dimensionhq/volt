from urllib.request import urlretrieve
import tarfile

urlretrieve(
    "https://registry.npmjs.org/swot-node/-/swot-node-2.0.147.tgz", "swot-node.tgz")
fname = 'swot-node.tgz'

tar = tarfile.open(fname, "r:gz")
tar.extractall(r"C:\Users\varun\.volt")
tar.close()
