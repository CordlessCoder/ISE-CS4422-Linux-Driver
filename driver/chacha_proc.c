#include "chacha.h"

static int lchacha_proc_show(struct seq_file* m, void* v) {
    seq_printf(m,
               "Sessions(Active):%08llu\n"
               "Sessions(Total): %08llu\n"
               "Reads:           %08llu\n"
               "Writes:          %08llu\n"
               "Ioctls:          %08llu\n"
               "Bytes Processed: %08llu\n"
               "Blocks:          %08llu\n"
               "Buffer Bytes:    %08llu\n"
               "Errors:          %08llu\n",
               (u64)atomic64_read(&lchacha_stats.active_sessions), (u64)atomic64_read(&lchacha_stats.total_sessions), (u64)atomic64_read(&lchacha_stats.reads),
               (u64)atomic64_read(&lchacha_stats.writes), (u64)atomic64_read(&lchacha_stats.ioctls), (u64)atomic64_read(&lchacha_stats.bytes_processed), (u64)atomic64_read(&lchacha_stats.blocks),
               (u64)atomic64_read(&lchacha_stats.current_buffer_bytes), (u64)atomic64_read(&lchacha_stats.errors));
    return 0;
#undef K
}

int lchacha_proc_open(struct inode* inode, struct file* file) { return single_open(file, lchacha_proc_show, NULL); }
