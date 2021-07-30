
import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { createSigner, keyring, api } = network;

const uri =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
    await api.isReady;

    const signer = createSigner(keyring.createFromUri(uri));

    // deploy math
    console.log('');
    console.log('Now deploy math contract');
    const mathContractFactory = await getContractFactory('math', signer);
    const balance1 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance1.toHuman());

    const mathContract = await mathContractFactory.deployed('new', {
        gasLimit: '200000000000',
        value:    '83500000000050000',
        salt: 'Coinversation Math'
    });
    console.log('Deploy math contract successfully.');
    console.log(
        'The contract address: ',
        mathContract.address.toString()
    );
    console.log(
        'The contract code hash: ',
        mathContract.abi.project.source.wasm.hash.toHex().toString()
    );

    // deploy base
    console.log('');
    console.log('Now deploy base contract');
    const baseContractFactory = await getContractFactory('base', signer);
    const balance2 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance2.toHuman());

    const baseContract = await baseContractFactory.deployed('new', mathContract.address, {
        gasLimit: '200000000000',
        value:    '83500000000050000',
        salt: 'Coinversation Base'
    });
    console.log('Deploy base contract successfully.');
    console.log(
        'The contract address: ',
        baseContract.address.toString()
    );
    console.log(
        'The contract code hash: ',
        baseContract.abi.project.source.wasm.hash.toHex().toString()
    );

    // deploy factory
    // deploy token
    console.log('');
    console.log('Now deploy token contract');
    const balance3 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance3.toHuman());

    const tokenFactory = await getContractFactory('token', signer);
    const tokenContract = await tokenFactory.deployed('new', mathContract.address, {
        gasLimit: '200000000000',
        value:    '83500000000050000',
        salt: 'Coinversation Token'
    });
    console.log('Deploy token contract successfully.');
    console.log(
        'The contract address: ',
        tokenContract.address.toString()
    );
    console.log(
        'The contract code hash: ',
        tokenContract.abi.project.source.wasm.hash.toHex().toString()

    );

    // deploy pool
    console.log('');
    console.log('Now deploy pool contract');
    const balance4 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance4.toHuman());

    const poolFactory = await getContractFactory('pool', signer);
    const poolContract = await poolFactory.deployed('new', mathContract.address, baseContract.address, tokenContract.address, {
        gasLimit: '200000000000',
        value:    '83500000000050000',
        salt: 'Coinversation Pool'
    });
    console.log('Deploy pool contract successfully.');
    console.log(
        'The contract address: ',
        poolContract.address.toString()
    );
    console.log(
        'The contract code hash: ',
        poolContract.abi.project.source.wasm.hash.toHex().toString()
    );

    console.log('');
    console.log('Now deploy factory contract');
    const balance5 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance5.toHuman());

    const contractFactory = await getContractFactory('factory', signer);
    const contract = await contractFactory.deployed('new', mathContract.address,
        baseContract.address, tokenContract.abi.project.source.wasm.hash.toHex(), poolContract.abi.project.source.wasm.hash.toHex(), {
        gasLimit: '200000000000',
        value:    '100000000000000000',
        salt: 'Coinversation Factory'
    });
    console.log('Deploy factory contract successfully.');
    console.log(
        'The contract address: ',
        contract.address.toString()
    );
    console.log(
        'The contract code hash: ',
        contract.abi.project.source.wasm.hash.toHex().toString()
    );
    const balance6 = await api.query.system.account(signer.address);
    console.log('Balance: ', balance6.toHuman());

    console.log('');
    console.log('######################################')
    console.log('Now we execute the contract!')

    const ts = parseInt((new Date().getTime()/1000).toString());
    console.log('Now timestamp is :', ts.toString())

    const txResponse = await contract.tx['newPool'](ts);

    console.log('factory,newPool the result is:', txResponse)

    api.disconnect();
}

run().catch((err) => {
    console.log(err);
});
