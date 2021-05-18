import couchdb


class DbManager:
    def __init__(self):
        self.db = None

    def initialise(self) -> couchdb.Database:
        server = couchdb.Server(
            url='http://xtremedevx:iliketurbo@localhost:5984')
        self.db = server['voltpkg']
        return server['voltpkg']

    def get(name, db) -> dict:
        return {'dependencies': db.get(name).get('dependencies')}
