;; Calculate factorial recursively
(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (dec n)))))
