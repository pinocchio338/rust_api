const { ensure, generateRandomBytes32, delay } = require("../util");

async function revokeRole(client, role) {
    const who = [...generateRandomBytes32()];
    ensure(!(await client.hasRole(role, who)));
    await client.grantRole(role, who);
    await delay(1000);
    
    await client.revokeRole(role, who);
    await delay(1000);
    ensure(!(await client.hasRole(role, who)));
}

async function renounceRole(client, role, who) {
    ensure(!(await client.hasRole(role, who)));
    await client.grantRole(role, who);
    await delay(1000);
    
    await client.renounceRole(role, who);
    await delay(1000);

    ensure(!(await client.hasRole(role, who)));
}

module.exports = {
    revokeRole, renounceRole
}
