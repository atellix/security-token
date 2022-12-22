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
    console.log('Security Token Program: ' + securityTokenPK.toString())
    const mint = anchor.web3.Keypair.generate()
    const owner = provider.wallet.publicKey
    const group = new PublicKey('DGzjPXnFFNw18FXSuMJVfwBThxBU2ohc2gAwsf2Z6FgA')
    const netauth = new PublicKey('AUTHXb39qs2VyztqH9zqh3LLLVGMzMvvYN3UXQHeJeEH')
    const approval = await programAddress([owner.toBuffer(), group.toBuffer()], netauth)
    var accountId1 = uuidv4()
    var accountBuf1 = Buffer.from(uuidparse(accountId1).reverse())
    //var accountId2 = uuidv4()
    //var accountBuf2 = Buffer.from(uuidparse(accountId2).reverse())
    const accountBytes = 209
    const accountRent = await provider.connection.getMinimumBalanceForRentExemption(accountBytes)
    const accountInfo1 = await programAddress([mint.publicKey.toBuffer(), owner.toBuffer(), accountBuf1], securityTokenPK)
    const account1 = new PublicKey(accountInfo1.pubkey)
    //const accountInfo2 = await programAddress([mint.publicKey.toBuffer(), owner.toBuffer(), accountBuf2], securityTokenPK)
    //const account2 = new PublicKey(accountInfo2.pubkey)

    console.log('Mint: ' + mint.publicKey.toString())

    const tx = new anchor.web3.Transaction()
    tx.add(
        anchor.web3.SystemProgram.transfer({
            fromPubkey: provider.wallet.publicKey,
            toPubkey: mint.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(257),
        })
    )
    tx.add(
        securityToken.instruction.createMint(
            6,
            'https://the.url/',
            {
                accounts: {
                    mint: mint.publicKey,
                    group: group,
                    netAuth: netauth,
                    manager: provider.wallet.publicKey,
                    feePayer: provider.wallet.publicKey,
                    systemProgram: SystemProgram.programId,
                }
            }
        )
    )
    tx.add(
        securityToken.instruction.createAccount(
            new anchor.BN(uuidparse(accountId1)),
            {
                accounts: {
                    account: account1,
                    mint: mint.publicKey,
                    owner: owner,
                    closeAuth: owner,
                    createAuth: new PublicKey(approval.pubkey),
                    systemProgram: SystemProgram.programId,
                }
            }
        ),
        securityToken.instruction.mint(
            new anchor.BN(1000 * 1000000),
            {
                accounts: {
                    mint: mint.publicKey,
                    manager: provider.wallet.publicKey,
                    to: account1,
                    toAuth: new PublicKey(approval.pubkey),
                },
            }
        ),
    )
    console.log(await provider.sendAndConfirm(tx, [mint]))
    let tokenAccount1 = await securityToken.account.securityTokenAccount.fetch(account1)
    console.log(showData(tokenAccount1))
    //let tokenAccount2 = await securityToken.account.securityTokenAccount.fetch(account2)
    //console.log(showData(tokenAccount2))
}

console.log('Begin')
main().then(() => console.log('Success')).catch(error => {
    console.log(error)
})
