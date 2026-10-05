import { User, Transfer, TransferRequest } from "@/types";

export const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3000"

const get = async <T = any>(endpoint: string, extraHeaders?: HeadersInit, omitCredentials?: boolean): Promise<T> => {
    const res = await (await fetch(API_URL + endpoint, {
        credentials: (omitCredentials ? "omit" : "include"),
        headers: extraHeaders,
        signal: AbortSignal.timeout(6000)
    })).json()

    if (!res.success) {
        throw new Error(res.message || "uknown error")
    }
    else {
        return res
    }
}

const withBody = async <T = any>(verb: string, endpoint: string, payload: any): Promise<T> => {
    const res = await (await fetch(API_URL + endpoint, {
        credentials: "include",
        method: verb,
        body: JSON.stringify(payload),
        headers: {
            "Content-Type": "application/json"
        }
    })).json()

    if (!res.success) {
        throw new Error(res.message || "uknown error")
    }
    else {
        return res
    }
}

const post = async <T = any>(endpoint: string, payload?: any): Promise<T> => {
    return await withBody<T>("post", endpoint, payload)
}

const put = async <T = any>(endpoint: string, payload?: any): Promise<T> => {
    return await withBody<T>("put", endpoint, payload)
}

const del = async <T = any>(endpoint: string, payload?: any): Promise<T> => {
    return await withBody<T>("delete", endpoint, payload)
}

// user

export async function getUser(): Promise<{ success: boolean; user: User }> {
    return await get<{ success: boolean; user: User }>("/user")
}

export async function onboard(onboardObj: any) {
    return await post("/user/onboard", onboardObj)
}

export async function getUserStorage() {
    return await get("/user/storage")
}

export async function putUserSettings(payload: any) {
    return await put("/user/settings", payload)
}

// auth

export async function login(email: string, password: string): Promise<{ success: boolean; token?: string; id?: string }> {
    return await post("/auth/login", { email, password })
}

export async function logout(): Promise<{ success: boolean }> {
    return await post("/auth/logout", {})
}

export async function register(email: string, password: string): Promise<{ success: boolean; token?: string; id?: string }> {
    return await post("/auth/register", { email, password })
}

export async function requestPasswordReset(email: string): Promise<{ success: boolean }> {
    return await post("/auth/passwordreset/request", { email })
}

export async function doPasswordReset(email: string, token: string, newPass: string): Promise<{ success: boolean }> {
    return await post("/auth/passwordreset/do", { email, token, newPass })
}

export async function doVerification(email: string, token: string): Promise<{ success: boolean }> {
    return await post("/auth/verification/do", { email, token })
}

// transfer

export async function getTransfer(id: string): Promise<{ success: boolean; transfer: Transfer }> {
    return await get<{ success: boolean; transfer: Transfer }>(`/transfer/${id}`)
}

export async function getTransferList(): Promise<{ success: boolean; transfers: Transfer[] }> {
    return await get<{ success: boolean; transfers: Transfer[] }>(`/transfer/list`)
}

export async function putTransfer(transferId: string, data: any) {
    return await put(`/transfer/${transferId}`, data)
}

export async function sendTransferByEmail(transferId: string, emails: string[]) {
    return await post(`/transfer/${transferId}/sendbyemail`, { emails })
}

export async function newTransfer(data: any): Promise<{ success: boolean; transfer: { secretCode: string } }> {
    return await post(`/transfer/new`, data)
}

export async function deleteTransfer(transferId: string): Promise<{ success: boolean }> {
    return await del(`/transfer/${transferId}`)
}

export const getTransferDownloadLink = (transfer: Transfer | null): string | null => {
    if (!transfer) return null
    if (typeof window === "undefined") return null
    return `${window.location.protocol}//${window.location.host}/transfer/${transfer.secretCode}`
}

export const getTransferAttachmentLink = (transfer: Transfer | null): string | null => {
    if (!transfer) return null
    return `${API_URL}/download/${transfer.secretCode}`
}

// transferrequest

export async function getTransferRequestList(): Promise<{ success: boolean; transferRequests: TransferRequest[] }> {
    return await get<{ success: boolean; transferRequests: TransferRequest[] }>(`/transferrequest/list`)
}

export async function newTransferRequest(data: any): Promise<{ success: boolean; transferRequest: TransferRequest }> {
    return await post(`/transferrequest/new`, data)
}

export async function sendTransferRequestByEmail(transferRequestId: string, emails: string[]) {
    return await post(`/transferrequest/${transferRequestId}/sendbyemail`, { emails })
}

export const getTransferRequestUploadLink = (transferRequest: TransferRequest | null): string | null => {
    if (!transferRequest) return null
    if (typeof window === "undefined") return null
    return `${window.location.protocol}//${window.location.host}/upload/${transferRequest.secretCode}`
}

export async function activateTransferRequest(transferRequestId: string): Promise<{ success: boolean }> {
    return await post(`/transferrequest/${transferRequestId}/activate`)
}

export async function deactivateTransferRequest(transferRequestId: string): Promise<{ success: boolean }> {
    return await post(`/transferrequest/${transferRequestId}/deactivate`)
}

// upload

export async function getUpload(secretCode: string): Promise<{ success: boolean; upload: Transfer }> {
    return await get<{ success: boolean; upload: Transfer }>(`/upload/${secretCode}`)
}

export async function markTransferComplete(secretCode: string): Promise<{ success: boolean }> {
    return await post(`/upload/${secretCode}/complete`, {})
}

// download

export async function registerTransferDownloaded(secretCode: string): Promise<{ success: boolean }> {
    return await post(`/download/${secretCode}/downloaded`)
}

// sign

export async function getUploadToken(secretCode: string): Promise<{ success: boolean; token: string }> {
    return await post(`/sign`, { secretCode, scope: "upload" })
}

export async function getDownloadToken(secretCode: string): Promise<{ success: boolean; token: string }> {
    return await post(`/sign`, { secretCode, scope: "download" })
}

// node

const nodePost = async <T = any>(nodeUrl: string, token: string, endpoint: string, payload: any): Promise<T> => {
    const res = await (await fetch(nodeUrl + endpoint, {
        credentials: "omit",
        method: "POST",
        body: JSON.stringify(payload),
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`
        }
    })).json()

    if (!res.success) {
        throw res
    }
    else {
        return res
    }
}

export async function signTransferDownload(nodeUrl: string, token: string): Promise<any> {
    return await nodePost(nodeUrl, token, "/download", {})
}