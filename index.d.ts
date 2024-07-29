/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface AouResponse {
  status?: number
  headers?: Record<String,String>
  body?: any
}
export interface AouOptions {
  json?: boolean
}
export type Request = AouRequest
export declare class AouRequest {
  context: any
  static fromString(request: string): Request
  get method(): string
  get path(): string
  get httpVersion(): string
  get headers(): Record<string, string>
  get body(): string
}
export declare class AouInstance { }
export declare class AouServer {
  constructor(options?: AouOptions | undefined | null)
  post(route:string,: handler:(request: AouRequest) => Promise<AouResponse>): void
  get(route:string,: handler:(request: AouRequest) => Promise<AouResponse>): void
  listen(host: string, port: number): Promise<AouInstance>
}
