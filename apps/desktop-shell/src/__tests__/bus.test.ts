import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { BusClient } from "../bus";

// Track instances created by the mock
const createdInstances: MockWebSocket[] = [];

class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.OPEN;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  onmessage: ((ev: { data: string }) => void) | null = null;

  send = vi.fn();
  close = vi.fn();

  constructor(public url: string) {
    createdInstances.push(this);
    setTimeout(() => this.onopen?.(), 0);
  }
}

const OriginalWebSocket = globalThis.WebSocket;
let connectCount = 0;

describe("BusClient", () => {
  let client: BusClient;

  beforeEach(() => {
    createdInstances.length = 0;
    connectCount = 0;
    vi.useFakeTimers();

    // Replace WebSocket with our mock and track calls
    const OrigMock = MockWebSocket;
    const TrackedMock = class extends OrigMock {
      constructor(url: string) {
        super(url);
        connectCount++;
      }
    } as unknown as typeof WebSocket;

    globalThis.WebSocket = TrackedMock;
    client = new BusClient("ws://localhost:3000");
  });

  afterEach(() => {
    client.dispose();
    globalThis.WebSocket = OriginalWebSocket;
    vi.useRealTimers();
  });

  it("connects to the WebSocket endpoint", () => {
    client.connect();
    expect(connectCount).toBe(1);
  });

  it("reports connected after open", () => {
    const onConnChange = vi.fn();
    client.setOnConnectionChange(onConnChange);
    client.connect();

    vi.advanceTimersByTime(10);

    expect(onConnChange).toHaveBeenCalledWith(true);
  });

  it("sends messages when connected", () => {
    client.connect();
    vi.advanceTimersByTime(10);
    client.send("test.command", { foo: "bar" });
    // send is called on the mock WebSocket
  });

  it("handles incoming messages", () => {
    const handler = vi.fn();
    client.onEvent(handler);
    client.connect();
    vi.advanceTimersByTime(10);

    const ws = createdInstances[0];
    ws.onmessage?.({ data: JSON.stringify({ type: "test.event", payload: { hello: "world" } }) });

    expect(handler).toHaveBeenCalledWith("test.event", { hello: "world" });
  });

  it("handles alternative frame format", () => {
    const handler = vi.fn();
    client.onEvent(handler);
    client.connect();
    vi.advanceTimersByTime(10);

    const ws = createdInstances[0];
    ws.onmessage?.({ data: JSON.stringify({ event: "app.launched", data: { app_id: "calc" } }) });

    expect(handler).toHaveBeenCalledWith("app.launched", { app_id: "calc" });
  });

  it("cleans up on dispose", () => {
    const onConnChange = vi.fn();
    client.setOnConnectionChange(onConnChange);
    client.connect();
    vi.advanceTimersByTime(10);
    client.dispose();
    expect(client.isConnected()).toBe(false);
  });

  it("connects without token in URL (auth via cookies)", () => {
    client.connect();
    const ws = createdInstances[0];
    expect(ws.url).not.toContain("token=");
  });
});
