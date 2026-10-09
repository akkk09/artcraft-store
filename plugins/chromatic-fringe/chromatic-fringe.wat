(module
  (memory (export "memory") 2)
  (data (i32.const 16) "{\"api\":1,\"id\":\"org.effectcraft.trokute.chromatic-fringe\",\"name\":\"Chromatic Fringe\",\"category\":\"Stylize\",\"version\":\"1.0.0\",\"author\":\"trokute\",\"description\":\"Offsets red and blue color channels in opposite directions.\",\"params\":[{\"id\":\"amount\",\"name\":\"Shift\",\"type\":\"slider\",\"default\":6,\"min\":0,\"max\":32,\"sliderMax\":16,\"decimals\":0},{\"id\":\"mix\",\"name\":\"Blend\",\"type\":\"slider\",\"default\":100,\"min\":0,\"max\":100,\"decimals\":0}]}\00")
  (func (export "ec_api_version") (result i32) i32.const 1)
  (func (export "ec_manifest_ptr") (result i32) i32.const 16)
  (func (export "ec_manifest_len") (result i32)
    (local $len i32)
    block $done
      loop $scan
        i32.const 16
        local.get $len
        i32.add
        i32.load8_u
        i32.eqz
        br_if $done
        local.get $len
        i32.const 1
        i32.add
        local.set $len
        br $scan
      end
    end
    local.get $len)
  (func (export "ec_alloc") (param $bytes i32) (result i32)
    (local $need i32)
    local.get $bytes
    i32.const 196607
    i32.add
    i32.const 16
    i32.shr_u
    memory.size
    i32.sub
    local.tee $need
    i32.const 0
    i32.gt_s
    if
      local.get $need
      memory.grow
      drop
    end
    i32.const 131072)
  (func (export "ec_render") (param $pixels i32) (param $width i32) (param $height i32) (param $params i32) (param $nparams i32) (param $time f64) (param $scale f64) (result i32)
    (local $shift i32)
    (local $mix f32)
    (local $row_bytes i32)
    (local $image_bytes i32)
    (local $scratch i32)
    (local $end i32)
    (local $pages i32)
    (local $row_start i32)
    (local $x i32)
    (local $y i32)
    (local $i i32)
    (local $red_x i32)
    (local $blue_x i32)
    (local $center i32)
    (local $output i32)
    (local $red_sample i32)
    (local $blue_sample i32)
    (local $alpha f32)
    (local $red_alpha f32)
    (local $blue_alpha f32)
    (local $red f32)
    (local $blue f32)
    (local $remain f32)
    local.get $nparams
    i32.const 2
    i32.lt_s
    if
      i32.const 1
      return
    end
    local.get $width
    i32.const 0
    i32.le_s
    local.get $height
    i32.const 0
    i32.le_s
    i32.or
    if
      i32.const 0
      return
    end
    local.get $params
    f64.load
    f64.const 0
    f64.max
    f64.const 32
    f64.min
    i32.trunc_sat_f64_s
    local.set $shift
    local.get $params
    f64.load offset=8
    f64.const 0
    f64.max
    f64.const 100
    f64.min
    f64.const 100
    f64.div
    f32.demote_f64
    local.set $mix
    local.get $shift
    i32.eqz
    local.get $mix
    f32.const 0
    f32.le
    i32.or
    if
      i32.const 0
      return
    end
    local.get $width
    i32.const 16
    i32.mul
    local.tee $row_bytes
    local.get $height
    i32.mul
    local.tee $image_bytes
    local.get $pixels
    i32.add
    local.tee $scratch
    local.get $row_bytes
    i32.add
    local.set $end
    local.get $end
    i32.const 65535
    i32.add
    i32.const 16
    i32.shr_u
    memory.size
    i32.sub
    local.tee $pages
    i32.const 0
    i32.gt_s
    if
      local.get $pages
      memory.grow
      i32.const -1
      i32.eq
      if
        i32.const 1
        return
      end
    end
    i32.const 0
    local.set $y
    block $rows_done
      loop $rows
        local.get $y
        local.get $height
        i32.ge_s
        br_if $rows_done
        local.get $pixels
        local.get $y
        local.get $row_bytes
        i32.mul
        i32.add
        local.set $row_start
        i32.const 0
        local.set $i
        block $copy_done
          loop $copy
            local.get $i
            local.get $row_bytes
            i32.ge_u
            br_if $copy_done
            local.get $scratch
            local.get $i
            i32.add
            local.get $row_start
            local.get $i
            i32.add
            i32.load
            i32.store
            local.get $i
            i32.const 4
            i32.add
            local.set $i
            br $copy
          end
        end
        i32.const 0
        local.set $x
        block $cols_done
          loop $cols
            local.get $x
            local.get $width
            i32.ge_s
            br_if $cols_done
            local.get $x
            local.get $shift
            i32.sub
            local.tee $red_x
            i32.const 0
            i32.lt_s
            if
              i32.const 0
              local.set $red_x
            end
            local.get $red_x
            local.set $red_x
            local.get $x
            local.get $shift
            i32.add
            local.tee $blue_x
            local.get $width
            i32.const 1
            i32.sub
            i32.gt_s
            if
              local.get $width
              i32.const 1
              i32.sub
              local.set $blue_x
            end
            local.get $blue_x
            local.set $blue_x
            local.get $scratch
            local.get $x
            i32.const 16
            i32.mul
            i32.add
            local.set $center
            local.get $row_start
            local.get $x
            i32.const 16
            i32.mul
            i32.add
            local.set $output
            local.get $scratch
            local.get $red_x
            i32.const 16
            i32.mul
            i32.add
            local.set $red_sample
            local.get $scratch
            local.get $blue_x
            i32.const 16
            i32.mul
            i32.add
            local.set $blue_sample
            local.get $center
            f32.load offset=12
            local.set $alpha
            local.get $alpha
            f32.const 0
            f32.gt
            if
              local.get $red_sample
              f32.load offset=12
              local.set $red_alpha
              local.get $blue_sample
              f32.load offset=12
              local.set $blue_alpha
              local.get $red_alpha
              f32.const 0
              f32.gt
              if (result f32)
                local.get $red_sample
                f32.load
                local.get $red_alpha
                f32.div
                local.get $alpha
                f32.mul
              else
                f32.const 0
              end
              local.set $red
              local.get $blue_alpha
              f32.const 0
              f32.gt
              if (result f32)
                local.get $blue_sample
                f32.load offset=8
                local.get $blue_alpha
                f32.div
                local.get $alpha
                f32.mul
              else
                f32.const 0
              end
              local.set $blue
              f32.const 1
              local.get $mix
              f32.sub
              local.set $remain
              local.get $output
              local.get $center
              f32.load
              local.get $remain
              f32.mul
              local.get $red
              local.get $mix
              f32.mul
              f32.add
              f32.store
              local.get $output
              local.get $center
              f32.load offset=8
              local.get $remain
              f32.mul
              local.get $blue
              local.get $mix
              f32.mul
              f32.add
              f32.store offset=8
            end
            local.get $x
            i32.const 1
            i32.add
            local.set $x
            br $cols
          end
        end
        local.get $y
        i32.const 1
        i32.add
        local.set $y
        br $rows
      end
    end
    i32.const 0))
