export interface Snapshot {
    id: string;
    url: string;
    timestamp: string;
    status_code: number;
    content_type: string;
}

export interface ResolveResponse {
    requested_at: string;
    actual_timestamp: string;
    replay_url: string;
}

export class ArchiveStream {
    private baseUrl: string;

    constructor(baseUrl: string = "http://localhost:3001") {
        this.baseUrl = `${baseUrl.replace(/\/$/, "")}/api/v1`;
    }

    async search(q: string): Promise<any[]> {
        const res = await fetch(`${this.baseUrl}/search?q=${encodeURIComponent(q)}`);
        return res.json();
    }

    async getSnapshots(url: string, limit: number = 50): Promise<Snapshot[]> {
        const res = await fetch(`${this.baseUrl}/snapshots?url=${encodeURIComponent(url)}&limit=${limit}`);
        return res.json();
    }

    async resolve(url: string, at: string): Promise<ResolveResponse> {
        const res = await fetch(`${this.baseUrl}/resolve?url=${encodeURIComponent(url)}&at=${at}`);
        return res.json();
    }

    async getDiff(url: string, from: string, to: string): Promise<any> {
        const res = await fetch(`${this.baseUrl}/diff?url=${encodeURIComponent(url)}&from=${from}&to=${to}`);
        return res.json();
    }

    async getSemantic(url: string, from: string, to: string): Promise<any> {
        const res = await fetch(`${this.baseUrl}/semantic?url=${encodeURIComponent(url)}&from=${from}&to=${to}`);
        return res.json();
    }

    async getTimeline(url: string): Promise<any> {
        const res = await fetch(`${this.baseUrl}/timeline?url=${encodeURIComponent(url)}`);
        return res.json();
    }
}
