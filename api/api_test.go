package api

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/confio/go-cosmwasm/types"
)

type Lookup struct {
	data map[string]string
}

func NewLookup() *Lookup {
	return &Lookup{data: make(map[string]string)}
}

func (l *Lookup) Get(key []byte) []byte {
	val := l.data[string(key)]
	return []byte(val)
}

func (l *Lookup) Set(key, value []byte) {
	l.data[string(key)] = string(value)
}

var _ KVStore = (*Lookup)(nil)

func TestInitAndReleaseCache(t *testing.T) {
	dataDir := "/foo"
	_, err := InitCache(dataDir, 3)
	require.Error(t, err)

	tmpdir, err := ioutil.TempDir("", "go-cosmwasm")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir)

	cache, err := InitCache(tmpdir, 3)
	require.NoError(t, err)
	ReleaseCache(cache)
}

func withCache(t *testing.T) (Cache, func()) {
	tmpdir, err := ioutil.TempDir("", "go-cosmwasm")
	require.NoError(t, err)
	cache, err := InitCache(tmpdir, 3)
	require.NoError(t, err)

	cleanup := func() {
		os.RemoveAll(tmpdir)
		ReleaseCache(cache)
	}
	return cache, cleanup
}

func TestCreateAndGet(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := ioutil.ReadFile("./testdata/contract.wasm")
	require.NoError(t, err)

	id, err := Create(cache, wasm)
	require.NoError(t, err)

	code, err := GetCode(cache, id)
	require.NoError(t, err)
	require.Equal(t, wasm, code)
}

func TestCreateFailsWithBadData(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm := []byte("some invalid data")
	_, err := Create(cache, wasm)
	require.Error(t, err)
}

func mockParams(signer []byte) types.Params {
	return types.Params{
		Block: types.BlockInfo{
		    Height: 123,
		    Time: 1578939743,
		    ChainID: "foobar",
		},
		Message: types.MessageInfo{
			Signer: signer,
			SentFunds: []types.Coin{{
				Denom:  "ATOM",
				Amount: "100",
			}},
		},
		Contract: types.ContractInfo{
			Address: binaryAddr("contract"),
			Balance: []types.Coin{{
				Denom:  "ATOM",
				Amount: "100",
			}},
		},
	}
}

func binaryAddr(human string) []byte {
    res := make([]byte, 42)
    copy(res, []byte(human))
    return res
}

func TestInstantiate(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	// create contract
	wasm, err := ioutil.ReadFile("./testdata/contract.wasm")
	require.NoError(t, err)
	id, err := Create(cache, wasm)
	require.NoError(t, err)

	// instantiate it with this store
	store := NewLookup()
	params, err := json.Marshal(mockParams(binaryAddr("creator")))
	require.NoError(t, err)
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)

	res, cost, err := Instantiate(cache, id, params, msg, store, 100000000)
	require.NoError(t, err)
    requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(87_748), cost)

	var resp types.CosmosResponse
	err = json.Unmarshal(res, &resp)
	require.NoError(t, err)
	require.Equal(t, "", resp.Err)
	require.Equal(t, 0, len(resp.Ok.Messages))
}

func TestHandle(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	id := createTestContract(t, cache)

	// instantiate it with this store
	store := NewLookup()
	params, err := json.Marshal(mockParams(binaryAddr("creator")))
	require.NoError(t, err)
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)

	start := time.Now()
	res, cost, err := Instantiate(cache, id, params, msg, store, 100000000)
	diff := time.Now().Sub(start)
	require.NoError(t, err)
    requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(87_748), cost)
	fmt.Printf("Time (87_433 gas): %s\n", diff)

	// execute with the same store
	params, err = json.Marshal(mockParams(binaryAddr("fred")))
	require.NoError(t, err)
	start = time.Now()
	res, cost, err = Handle(cache, id, params, []byte(`{}`), store, 100000000)
	diff = time.Now().Sub(start)
	require.NoError(t, err)
    requireOkResponse(t, res, 1)
	assert.Equal(t, uint64(131_864), cost)
	fmt.Printf("Time (131_864 gas): %s\n", diff)
}

func TestMultipleInstances(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	id := createTestContract(t, cache)

	// instance1 controlled by fred
	store1 := NewLookup()
	params, err := json.Marshal(mockParams(binaryAddr("regen")))
	require.NoError(t, err)
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	res, cost, err := Instantiate(cache, id, params, msg, store1, 100000000)
	require.NoError(t, err)
    requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(87_115), cost)

	// instance2 controlled by mary
	store2 := NewLookup()
	params, err = json.Marshal(mockParams(binaryAddr("chorus")))
	require.NoError(t, err)
	msg = []byte(`{"verifier": "mary", "beneficiary": "sue"}`)
	res, cost, err = Instantiate(cache, id, params, msg, store2, 100000000)
	require.NoError(t, err)
    requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(86_493), cost)

	// fail to execute store1 with mary
	resp := exec(t, cache, id, "mary", store1, 117_200)
	require.Equal(t, "Unauthorized", resp.Err)

	// succeed to execute store1 with fred
	resp = exec(t, cache, id, "fred", store1, 131_594)
	require.Equal(t, "", resp.Err)
	require.Equal(t, 1, len(resp.Ok.Messages))

	// succeed to execute store2 with mary
	resp = exec(t, cache, id, "mary", store2, 131_442)
	require.Equal(t, "", resp.Err)
	require.Equal(t, 1, len(resp.Ok.Messages))
}

func requireOkResponse(t *testing.T, res []byte, expectedMsgs int) {
    var resp types.CosmosResponse
	err := json.Unmarshal(res, &resp)
	require.NoError(t, err)
	require.Equal(t, "", resp.Err)
	require.Equal(t, expectedMsgs, len(resp.Ok.Messages))
}

func createTestContract(t *testing.T, cache Cache) []byte {
	wasm, err := ioutil.ReadFile("./testdata/contract.wasm")
	require.NoError(t, err)
	id, err := Create(cache, wasm)
	require.NoError(t, err)
	return id
}

// exec runs the handle tx with the given signer
func exec(t *testing.T, cache Cache, id []byte, signer string, store KVStore, gas uint64) types.CosmosResponse {
	params, err := json.Marshal(mockParams(binaryAddr(signer)))
	require.NoError(t, err)
	res, cost, err := Handle(cache, id, params, []byte(`{}`), store, 100000000)
	require.NoError(t, err)
	assert.Equal(t, gas, cost)

	var resp types.CosmosResponse
	err = json.Unmarshal(res, &resp)
	require.NoError(t, err)
	return resp
}

func TestQuery(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	id := createTestContract(t, cache)

	// set up contract
	store := NewLookup()
	params, err := json.Marshal(mockParams(binaryAddr("creator")))
	require.NoError(t, err)
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	_, _, err = Instantiate(cache, id, params, msg, store, 100000000)
	require.NoError(t, err)

	// invalid query
	query := []byte(`{"Raw":{"val":"config"}}`)
	data, _, err := Query(cache, id, query, store, 100000000)
	require.NoError(t, err)
	var badResp types.QueryResponse
	err = json.Unmarshal(data, &badResp)
	require.NoError(t, err)
	require.Equal(t, badResp.Err, "Error parsing QueryMsg: unknown variant `Raw`, expected `raw`")

	// make a valid query
	query = []byte(`{"raw":{"key":"config"}}`)
	data, _, err = Query(cache, id, query, store, 100000000)
	require.NoError(t, err)
	var resp types.QueryResponse
	err = json.Unmarshal(data, &resp)
	require.NoError(t, err)
	require.Empty(t, resp.Err)
	require.Equal(t, 1, len(resp.Ok.Results))
	model := resp.Ok.Results[0]
	require.Equal(t, "config", model.Key)
	require.Equal(t, `{"verifier":"fred","beneficiary":"bob","funder":"creator"}`, string(model.Value))
}
