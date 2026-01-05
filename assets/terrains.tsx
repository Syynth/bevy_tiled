<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.2" name="terrains" tilewidth="16" tileheight="16" tilecount="124" columns="4">
 <image source="terrains.png" width="64" height="496"/>
 <tile id="1">
  <objectgroup draworder="index" id="2">
   <object id="1" x="16" y="0">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings"/>
    </properties>
    <polygon points="0,0 0,16 -16,0"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="2">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings"/>
    </properties>
    <polygon points="0,0 16,8 16,0"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="3">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings"/>
    </properties>
    <polygon points="16,0 0,0 0,8 16,16"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="4">
  <properties>
   <property name="collider" type="class" propertytype="avian::PhysicsSettings"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0" width="16" height="16">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings">
      <properties>
       <property name="is_sensor" type="bool" value="false"/>
       <property name="lock_rotation" type="bool" value="false"/>
      </properties>
     </property>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="5">
  <objectgroup draworder="index" id="2">
   <object id="1" x="16" y="8">
    <polygon points="0,0 -16,8 -16,-8 0,-8"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="6">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0">
    <polygon points="0,0 16,8 16,0"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="8">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0">
    <polygon points="0,0 0,8 16,16 16,0"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="9">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0">
    <polygon points="0,0 16,8 16,16 0,16"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="10">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="8">
    <polygon points="0,0 16,8 0,8"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="11">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="16">
    <polygon points="0,0 16,-8 16,0"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="12">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="8">
    <polygon points="0,0 16,-8 16,8 0,8"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="49">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0.165169" y="0.330338">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings">
      <properties>
       <property name="body_type" propertytype="avian::BodyType" value="Static"/>
      </properties>
     </property>
    </properties>
    <polygon points="0,0 15.6911,15.0304 0.165169,15.3607"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="76">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0" width="16" height="16">
    <properties>
     <property name="collider" type="class" propertytype="avian::PhysicsSettings">
      <properties>
       <property name="body_type" propertytype="avian::BodyType" value="Dynamic"/>
      </properties>
     </property>
    </properties>
   </object>
  </objectgroup>
 </tile>
</tileset>
