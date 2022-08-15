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

async function main() {
    console.log('Security Token Program: ' + securityTokenPK.toString())
    const mint = anchor.web3.Keypair.generate()
    const owner = provider.wallet.publicKey
    const group = new PublicKey('91Q2u3RvAp64qB9H84gFnUmwkT1s4MZSXWxu7PMZ6Wre')
    const netauth = new PublicKey('AUTHXb39qs2VyztqH9zqh3LLLVGMzMvvYN3UXQHeJeEH')
    const approval = await programAddress([owner.toBuffer(), group.toBuffer()], netauth)
    var accountId1 = uuidv4()
    var accountBuf1 = Buffer.from(uuidparse(accountId1).reverse())
    var accountId2 = uuidv4()
    var accountBuf2 = Buffer.from(uuidparse(accountId2).reverse())
    const accountBytes = 209
    const accountRent = await provider.connection.getMinimumBalanceForRentExemption(accountBytes)
    const accountInfo1 = await programAddress([mint.publicKey.toBuffer(), owner.toBuffer(), accountBuf1], securityTokenPK)
    const account1 = new PublicKey(accountInfo1.pubkey)
    const accountInfo2 = await programAddress([mint.publicKey.toBuffer(), owner.toBuffer(), accountBuf2], securityTokenPK)
    const account2 = new PublicKey(accountInfo2.pubkey)

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
            0,
            'https://the.url/',
            {
                accounts: {
                    mint: mint.publicKey,
                    group: group,
                    netAuth: netauth,
                    manager: provider.wallet.publicKey,
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
        securityToken.instruction.createAccount(
            new anchor.BN(uuidparse(accountId2)),
            {
                accounts: {
                    account: account2,
                    mint: mint.publicKey,
                    owner: owner,
                    closeAuth: owner,
                    createAuth: new PublicKey(approval.pubkey),
                    systemProgram: SystemProgram.programId,
                }
            }
        ),
        securityToken.instruction.mint(
            new anchor.BN(1000),
            {
                accounts: {
                    mint: mint.publicKey,
                    manager: provider.wallet.publicKey,
                    to: account1,
                    toAuth: new PublicKey(approval.pubkey),
                },
            }
        ),
        securityToken.instruction.transfer(
            new anchor.BN(500),
            {
                accounts: {
                    user: provider.wallet.publicKey,
                    from: account1,
                    fromAuth: new PublicKey(approval.pubkey),
                    to: account2,
                    toAuth: new PublicKey(approval.pubkey),
                },
            }
        )
    )
    console.log(await provider.sendAndConfirm(tx, [mint]))
}

console.log('Begin')
main().then(() => console.log('Success')).catch(error => {
    console.log(error)
})
