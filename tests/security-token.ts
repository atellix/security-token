import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { SecurityToken } from '../target/types/security_token';

describe('security-token', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.SecurityToken as Program<SecurityToken>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
