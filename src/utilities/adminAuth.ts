import { Request } from "express";
import * as crypto from "node:crypto";
import { isRedisConnected } from "../redis";


export default class AdminAuthManager {

    //#region Properties
        private _token: string;
    //#endregion

    //#region Constructor
        constructor(token: string)
        {
            if (token == null || token.trim() == "")
                throw new Error("ERROR: token is required to calculate the OTP");

            this._token = token;
        }
    //#endregion

    //#region Getter / Setter
    //#endregion


    private base32ToBuffer(str: string): string {
        // Tabella di decodifica base32
        const base32Chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
        let bits = '';
        
        // Converti ogni carattere in 5 bit
        str.split('').forEach(char => {
            const value = base32Chars.indexOf(char);
            if (value === -1) {
                throw new Error('Invalid base32 character: ' + char);
            }
            bits += value.toString(2).padStart(5, '0');
        });

        // Converti i bit in bytes
        const bytes = new Uint8Array(Math.floor(bits.length / 8));
        for (let i = 0; i < bytes.length; i++) {
            bytes[i] = parseInt(bits.substr(i * 8, 8), 2);
        }

        return Buffer.from(bytes).toString('utf-8');
    }

    public matchOTP(givenOTP: string): boolean {
        const timeStep = 30;
        const currentTime = Math.floor(Date.now() / 1000);
        const window = 2
        
        
        for (let i = -window; i <= window; i++) {
            let testTime = Math.floor((currentTime + (i * timeStep)) / timeStep);
            
            const timeBuffer = Buffer.alloc(8);
            for (let j = timeBuffer.length - 1; j >= 0; j--) {
                timeBuffer[j] = testTime & 0xff;
                testTime = testTime >> 8;  // Corretto l'operatore di shift
            }

            const hmac = crypto.createHmac('sha1', this.base32ToBuffer(this._token));
            hmac.update(timeBuffer);
            const hmacResult = hmac.digest();

            const offset = hmacResult[hmacResult.length - 1] & 0xf;
            const code = ((hmacResult[offset] & 0x7f) << 24) |
                        ((hmacResult[offset + 1] & 0xff) << 16) |
                        ((hmacResult[offset + 2] & 0xff) << 8) |
                        (hmacResult[offset + 3] & 0xff);

            const testOTP = (code % 1000000).toString().padStart(6, '0');
            if (givenOTP === testOTP) {
                return true;
            }
        }
        
        return false;
    }

    //#endregion
    
}