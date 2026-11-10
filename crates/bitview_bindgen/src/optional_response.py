# Runs with the proposed runtime and a FixtureClient supplied by the generator test.
passed = 0
class MockResponse:
    def __init__(self, status, data): self.status, self.data = status, data
    def read(self): return self.data
class MockConnection:
    def __init__(self): self.replies, self.calls = [], []
    def request(self, method, path, body=None, headers=None): self.calls.append((method,path,body,headers))
    def getresponse(self): return self.replies.pop(0)
    def close(self): pass
client = FixtureClient('http://offline.invalid')
connection = MockConnection()
client._conn = connection

def check(name, run):
    global passed
    run()
    passed += 1
    print('PASS ' + name)
def eq(actual, expected): assert actual == expected, (actual, expected)
def response(status, body, method='GET', optional=True):
    connection.replies.append(MockResponse(status, body))
    if method == 'POST': return client.post_optional('body')
    return client.get_optional() if optional else client.get_ordinary()
def fails(status, body, expected, method='GET', optional=True):
    try: response(status, body, method, optional)
    except expected as error: return error
    raise AssertionError('expected ' + expected.__name__)
check('204 bypasses JSON decoder',lambda:eq(response(204,b''),None))
check('200 object decoded',lambda:eq(response(200,b'{"tx_id":"x","rates":{"keep_key":1}}'),{'tx_id':'x','rates':{'keep_key':1}}))
check('empty 200 fails',lambda:fails(200,b'',json.JSONDecodeError))
check('malformed 200 fails',lambda:fails(200,b'{',json.JSONDecodeError))
check('JSON null 200 remains valid JSON',lambda:eq(response(200,b'null'),None))
for status in [400,404,429,503]:
    def error_case(status=status):
        error=fails(status,b'{"error":{"code":"invalid_parameter","message":"detail"}}',BitviewError)
        eq((error.status,error.code,str(error)),(status,'invalid_parameter','detail'))
    check(str(status)+' error details preserved',error_case)
check('ordinary JSON 200 unchanged',lambda:eq(response(200,b'{"tx_id":"normal"}',optional=False),{'tx_id':'normal'}))
check('ordinary empty 200 fails',lambda:fails(200,b'',json.JSONDecodeError,optional=False))
check('ordinary undeclared 204 fails',lambda:fails(204,b'',json.JSONDecodeError,optional=False))
check('POST 204 bypasses JSON decoder',lambda:eq(response(204,b'',method='POST'),None))
check('POST 200 decoded',lambda:eq(response(200,b'{"tx_id":"post"}',method='POST'),{'tx_id':'post'}))
check('POST empty 200 fails',lambda:fails(200,b'',json.JSONDecodeError,method='POST'))
check('POST error details preserved',lambda:eq(fails(503,b'{"error":{"code":"state_updating","message":"wait"}}',BitviewError,method='POST').status,503))
def no_json_parse():
    original_loads=json.loads
    try:
        def forbidden(*args,**kwargs): raise AssertionError('204 parsed')
        json.loads=forbidden
        eq(response(204,b''),None)
        eq(response(204,b'',method='POST'),None)
    finally: json.loads=original_loads
check('204 read never calls json.loads',no_json_parse)
check('no hidden retry, one dispatch per call',lambda:eq(len(connection.calls),18))
assert not connection.replies
print(f'FINISHED {passed} Python mock-HTTP runtime cases')
