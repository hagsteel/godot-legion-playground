extends Node

var gdn

func _ready():
    gdn = GDNative.new()
    var status = false

    gdn.library = load("res://libplayground.gdnlib")

    if gdn.initialize():
        status = gdn.call_native("standard_varcall", "run_tests", [])

        if status:
            print('all tests passed')
        else:
            print('test failure')

        gdn.terminate()
        get_tree().quit(1)