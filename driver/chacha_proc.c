#include "chacha.h"

static int lchacha_proc_show(struct seq_file* m, void* v) {
    seq_printf(m,
               "Sessions(Active): %08llu\n"
               "Sessions(Total):  %08llu\n"
               "Bytes:            %08llu\n",
               (u64)atomic64_read(&lchacha_active_sessions), (u64)atomic64_read(&lchacha_total_sessions), (u64)atomic64_read(&lchacha_bytes_processed));
    return 0;
#undef K
}

int lchacha_proc_open(struct inode* inode, struct file* file) { return single_open(file, lchacha_proc_show, NULL); }
