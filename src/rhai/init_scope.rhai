let client = #{
    global: #{
        set: |key,value| Storage.set(key, value),
        get: |key| Storage.get(key),
        isEmpty: || Storage.is_empty(),
        clear: |key| Storage.clear(key),
        clearAll: || Storage.clear_all(),
    },
};

let console = #{
    log: |value| print(value),
};
