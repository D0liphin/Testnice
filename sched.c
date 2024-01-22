struct sched_entity {
    /* 
     * { unsigned long weight, u32 inv_weight } 
     */
    struct load_weight              load;
    /*
     * {
     *     unsigned long  __rb_parent_color;
     *     struct rb_node *rb_right;
     *     struct rb_node *rb_left;
     * }
    */
    struct rb_node                  run_node;
    u64                             deadline;
    u64                             min_deadline;
            
    /* { struct list_head *next, *prev; } */
    struct list_head                group_node;
    unsigned int                    on_rq;
            
    u64                             exec_start;
    u64                             sum_exec_runtime;
    u64                             prev_sum_exec_runtime;
    /* 
     * Per-thread measure of 'runtime'. Lower means "more deserving of runtime".
     * Is a function of the priority, ni and more. 
     * Ni of 0 means vruntime is equal to physical runtime.
    */
    u64                             vruntime;
    s64                             vlag;
    u64                             slice;
            
    u64                             nr_migrations;

#ifdef CONFIG_FAIR_GROUP_SCHED
    int                             depth;
    struct sched_entity             *parent;
    /* rq on which this entity is (to be) queued: */
    struct cfs_rq                   *cfs_rq;
    /* rq "owned" by this entity/group: */
    struct cfs_rq                   *my_q;
    /* cached value of my_q->h_nr_running */
    unsigned long                   runnable_weight;
#endif

#ifdef CONFIG_SMP
    /*
     * Per entity load average tracking.
     *
     * Put into separate cache line so it does not
     * collide with read-mostly values above.
     */
    struct sched_avg                avg;
#endif
};