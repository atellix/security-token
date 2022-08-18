const { Buffer } = require('buffer')
//const { DateTime } = require('luxon')
const { v4: uuidv4, parse: uuidparse } = require('uuid')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js')
//const { promisify } = require('util')
//const exec = promisify(require('child_process').exec)
//const fs = require('fs').promises
//const base32 = require('base32.js')
const anchor = require('@project-serum/anchor')
const { associatedTokenAddress, programAddress, importSecretKey, exportSecretKey, jsonFileRead, jsonFileWrite } = require('../../js/atellix-common')

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)
const securityToken = anchor.workspace.SecurityToken
const securityTokenPK = securityToken.programId

function showData(spec) {
    var r = {}
    for (var i in spec) {
        if (typeof spec[i] === 'object' && spec[i].constructor.name === 'Object') {
            r[i] = showData(spec[i])
        } else if (typeof spec[i].toString !== 'undefined') {
            r[i] = spec[i].toString()
        }
    }
    return r
}

async function main() {
    let account1 = new PublicKey(process.argv[2])
    let tokenAccount1 = await securityToken.account.securityTokenAccount.fetch(account1)
    console.log(showData(tokenAccount1))
}

console.log('Begin')
main().then(() => console.log('Success')).catch(error => {
    console.log(error)
})
