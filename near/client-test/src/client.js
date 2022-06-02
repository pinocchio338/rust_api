const { toBuffer, bufferU64BE, encodeData } = require("./util");

class DapiServer {
    contract;

    constructor(contract) {
        this.contract = contract;
    }

    async hasRole(role, who) {
        return await this.contract.has_role({ role: [...role], who: [...who]});
    }

    async grantRole(role, who) {
        return await this.contract.grant_role({ role: [...role], who: [...who]});
    }

    async renounceRole(role, who) {
        return await this.contract.renounce_role( { args: { role: [...role], who: [...who]} });
    }

    async updateBeaconWithSignedData(airnodeAddress, templateId, timestamp, data, signature) {
        const pubKeyBuf = toBuffer(airnodeAddress);
        const bufferedTimestamp = bufferU64BE(timestamp);
        const bufferedTemplateId = Buffer.from(templateId, 'hex');
        const encodedData = encodeData(data);
        const buf = toBuffer(signature);

        await this.contract.update_beacon_with_signed_data(
            {
                args: {
                  airnode: [...pubKeyBuf],
                  template_id: [...bufferedTemplateId],
                  timestamp: [...bufferedTimestamp],
                  data: [...encodedData],
                  signature: [...buf]
                }
            }
        );
    }

    async updateBeaconSetWithBeacons(beaconIds) {
        await this.contract.update_dapi_with_beacons(
            {
                args: {
                    beacon_ids: beaconIds
                }
            }
        );
    }

    async setDapiName(name, datapointId) {
        await this.contract.set_name(
            {
                args: {
                    name: [...name],
                    datapoint_id: [...datapointId]
                }
            }
        );
    }

    async deriveBeaconId(airnode, templateId) {
        return await this.contract.derive_beacon_id(
            {
                airnode: [...airnode],
                template_id: [...templateId]
            }
        );
    }

    async deriveBeaconSetId(beaconIds) {
        return await this.contract.derive_beacon_set_id(
            {
                beacon_ids: beaconIds
            }
        );
    }

    async dapiNameToDataFeedId(name) {
        return await this.contract.name_to_data_point_id(
            {
                name: [...name]
            }
        );
    }
}

module.exports = { DapiServer }