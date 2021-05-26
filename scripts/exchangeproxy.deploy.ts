import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
    await api.isReady;

    const signer = createSigner(keyring.createFromUri(uri));
    const contractFactory = await getContractFactory('exchangeproxy', signer);

    const balance = await api.query.system.account(signer.address);
    console.log('Balance: ', balance.toHuman());

    const cusdContractFactory = await getContractFactory('pat_standard', signer);
    const cusdContract = await cusdContractFactory.deployed('IPat,new', '0', 'Coinversation USD Token', 'Cusd', '6', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Cusd Token'
    });
    console.log(
        'Deploy Cusd successfully. The contract address: ',
        cusdContract.address.toString()
    );
    console.log('');

    const cdotContractFactory = await getContractFactory('pat_standard', signer);
    const cdotContract = await cdotContractFactory.deployed('IPat,new', '10000000000000000', 'Coinversation Polkadot Token', 'Cdot', '10', {
        gasLimit: '200000000000',
        value: '0',
        salt: 'Coinversation DOT Token'
    });
    console.log(
        'Deploy Cdot successfully. The contract address: ',
        cdotContract.address.toString()
    );
    console.log('');

    const contract = await contractFactory.deployed('new', cusdContract.address, cdotContract.address,{
        gasLimit: '200000000000',
        value: '0',
        salt: 'ExchangeProxy'
    });
    console.log(
        'Deploy exchangeproxy ( ExchangeProxy ) successfully. The contract address: ',
        contract.address.toString()
    );

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
