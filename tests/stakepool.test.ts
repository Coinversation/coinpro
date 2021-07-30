import Contract from '@redspot/patract/contract';
import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getAddresses, getSigners } = network;

let signer: string;
let cto: Contract;
let cusd: Contract;

describe('STAKEPOOL', () => {
  after(() => {
    return api.disconnect();
  });

  before(async () => {
    await api.isReady;
    const signerAddresses = await getAddresses();
    signer = signerAddresses[0];
  });

  it('deploy cto token', async () => {
    const contractFactory = await getContractFactory('pat_standard', signer);
    cusd = await contractFactory.deploy('IPat,new', '0', 'Coinversation USD Token', 'Cusd', '6');
    expect(cusd.address).to.exist;
  });
});
