package com.kushtrimh.utils;

import java.util.HashMap;
import java.util.Map;

/**
 * @author Kushtrim Hajrizi
 */
public class MutableMapFactory {

    public static Map<String, Object> of(Object... params) {
        if (params.length % 2 != 0) {
            throw new IllegalArgumentException("Invalid number of arguments");
        }
        Map<String, Object> map = new HashMap<>();
        for (int i = 0; i < params.length; i += 2) {
            map.put((String) params[i], params[i + 1]);
        }
        return map;
    }
}
