console.log("in config.js");

const CONTRACT_NAME = process.env.CONTRACT_NAME || 'mochacha.testnet'; /* TODO: change this to your account */
console.log("contract name: ", CONTRACT_NAME);


function getConfig (env) {
  console.log("env: ", env);
  switch (env) {
    case 'production':
    case 'mainnet':
      return {
        networkId: 'mainnet',
        nodeUrl: 'https://rpc.mainnet.near.org',
        contractName: CONTRACT_NAME,
        walletUrl: 'https://wallet.near.org',
        helperUrl: 'https://helper.mainnet.near.org'
      }
    case 'development':
    case 'test':
    case 'testnet':
      return {
        networkId: 'testnet',
        nodeUrl: 'https://rpc.testnet.near.org',
        contractName: CONTRACT_NAME,
        walletUrl: 'https://wallet.testnet.near.org',
        helperUrl: 'https://helper.testnet.near.org',
        // masterAccount: CONTRACT_NAME
      }
    default:
      throw Error(`Unconfigured environment '${env}'. Can be configured in src/config.js.`)
  }
}

module.exports = getConfig
