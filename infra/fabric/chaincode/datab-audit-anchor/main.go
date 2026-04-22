package main

import (
	"log"

	"github.com/hyperledger/fabric-contract-api-go/v2/contractapi"

	"datab.local/fabric-chaincode/datab-audit-anchor/chaincode"
)

func main() {
	anchorChaincode, err := contractapi.NewChaincode(&chaincode.AnchorContract{})
	if err != nil {
		log.Panicf("create datab-audit-anchor chaincode: %v", err)
	}

	if err := anchorChaincode.Start(); err != nil {
		log.Panicf("start datab-audit-anchor chaincode: %v", err)
	}
}
