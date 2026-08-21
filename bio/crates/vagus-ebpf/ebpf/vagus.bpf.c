// vagus.bpf.c — kernel-side vagus nerve probes (aya-ebpf).
//
// Build: cargo xtask build-ebpf  (or clang -target bpf -O2 -g -c vagus.bpf.c -o vagus.bpf.o)
// Loaded by crates/vagus-ebpf/src/ebpf_loader.rs on Linux.
//
// Emits RawEvent records into the VAGUS_EVENTS ring buffer:
//   struct RawEvent { u8 kind; u32 severity_milli; char detail[56]; u8 detail_len; }
// kind: 0 = MemoryPressure, 1 = IoCongestion, 2 = NetworkAnomaly

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "GPL";

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} VAGUS_EVENTS SEC(".maps");

struct raw_event {
    __u8 kind;
    __u32 severity_milli;
    char detail[56];
    __u8 detail_len;
};

static __always_inline void emit(__u8 kind, __u32 severity, const char *detail)
{
    struct raw_event *e = bpf_ringbuf_reserve(&VAGUS_EVENTS, sizeof(*e), 0);
    if (!e)
        return;
    e->kind = kind;
    e->severity_milli = severity;
    e->detail_len = bpf_probe_read_kernel_str(e->detail, sizeof(e->detail), detail);
    bpf_ringbuf_submit(e, 0);
}

// ── Memory pressure: page reclaim activity ──
SEC("tracepoint/vmscan/mm_vmscan_direct_reclaim_begin")
int reclaim_begin(struct trace_event_raw_mm_vmscan_direct_reclaim_begin *ctx)
{
    emit(0, 850, "direct reclaim begin");
    return 0;
}

// ── I/O congestion: block request latency ──
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 10240);
    __type(key, __u64);
    __type(value, __u64);
} rq_start SEC(".maps");

SEC("tracepoint/block/block_rq_issue")
int rq_issue(struct trace_event_raw_block_rq *ctx)
{
    __u64 key = (__u64)ctx->dev << 32 | ctx->sector;
    __u64 ts = bpf_ktime_get_ns();
    bpf_map_update_elem(&rq_start, &key, &ts, BPF_ANY);
    return 0;
}

SEC("tracepoint/block/block_rq_complete")
int rq_complete(struct trace_event_raw_block_rq *ctx)
{
    __u64 key = (__u64)ctx->dev << 32 | ctx->sector;
    __u64 *start = bpf_map_lookup_elem(&rq_start, &key);
    if (!start)
        return 0;
    __u64 latency_ms = (bpf_ktime_get_ns() - *start) / 1000000;
    bpf_map_delete_elem(&rq_start, &key);
    if (latency_ms > 100) {
        __u32 severity = latency_ms > 1000 ? 1000 : (__u32)latency_ms;
        emit(1, severity, "block rq slow");
    }
    return 0;
}

// ── Network anomaly: TCP retransmits ──
SEC("kprobe/tcp_retransmit_skb")
int retransmit(struct pt_regs *ctx)
{
    emit(2, 600, "tcp retransmit");
    return 0;
}
