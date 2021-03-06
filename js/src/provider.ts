import Axios from "axios";
import { SubscriptionCheckResponse, GrantedTokensResponse } from "./types";
import * as zksync from "zksync";

export class ApiError extends Error {
    constructor(message: string, public apiError: ApiErrorObject) {
        super(message);
    }
}

export interface ApiErrorObject {
    error: string;
}

export class HTTPTransport {
    public constructor(public address: string) {
    }

    public endpoint(postfix: string): string {
        return this.address + "api/v0.1" + postfix;
    }

    // JSON RPC request
    async request(endpoint: string, request = null): Promise<any> {
        const response = await Axios.post(endpoint, request).then(resp => {
            return resp;
        });

        if (response.status == 200) {
            return response.data;
        } else if (response.data.error) {
            throw new ApiError("API response error", response.data.error);
        } else {
            throw new Error(`Unknown API Error: ${JSON.stringify(response)}`);
        }
    }
}

export class Provider {
    transport: HTTPTransport;

    public constructor(public address: string) {
        this.transport = new HTTPTransport(address);
    }

    async genesisWalletAddress(): Promise<zksync.types.Address> {
        let endpoint = this.transport.endpoint("/genesis_wallet_address");
        let response = await this.transport.request(endpoint);
        return response.address;
    }

    async isUserSubscribed(user: string, communityName: string): Promise<SubscriptionCheckResponse> {
        let endpoint = this.transport.endpoint("/is_user_subscribed");
        return await this.transport.request(endpoint, {
            user,
            communityName
        });
    }

    async grantedTokens(user: string, communityName: string): Promise<GrantedTokensResponse> {
        let endpoint = this.transport.endpoint("/granted_tokens");
        return await this.transport.request(endpoint, {
            user,
            communityName
        });
    }

    async getMintingSignature(user: string, communityName: string, mintingTx: zksync.types.TransferFrom): Promise<zksync.types.Signature> {
        let endpoint = this.transport.endpoint("/get_minting_signature");
        let response = await this.transport.request(endpoint, {
            user,
            communityName,
            mintingTx
        });

        return response.signature.zksyncSignature;
    }

    async subscribe(user: string, communityName: string, subscriptionWallet: zksync.types.Address, txs: zksync.types.SubscriptionTx[]) {
        let endpoint = this.transport.endpoint("/subscribe");
        await this.transport.request(endpoint, {
            user,
            communityName,
            subscriptionWallet,
            txs
        });
    }
}