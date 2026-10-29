{ ... }: {
  users.users.user = {
    isNormalUser = true;
    password = "user";
    extraGroups = [ "wheel" ];
  };
  users.users.root.password = "root";
  services.getty.autologinUser = "user";
}
