const { generateRandomBytes32, delay } = require("../../src/util");

async function revokeRole(client, role) {
    const who = generateRandomBytes32().toString();
    await expect(client.hasRole(role, who)).resolves.toEqual(false);
    await client.grantRole(role, who);
    await delay(1000);
    await expect(client.hasRole(role, who)).resolves.toEqual(true);
    await client.revokeRole(role, who);
    await delay(1000);
    await expect(client.hasRole(role, who)).resolves.toEqual(false);
}

async function renounceRole(client, role, who) {
    await expect(client.hasRole(role, who)).resolves.toEqual(false);
    await client.grantRole(role, who);
    await delay(1000);
    await expect(client.hasRole(role, who)).resolves.toEqual(true);
    await client.renounceRole(role, who);
    await delay(1000);
    await expect(client.hasRole(role, who)).resolves.toEqual(false);
}

module.exports = {
    revokeRole, renounceRole
}
